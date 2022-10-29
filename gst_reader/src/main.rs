//! Read GStreamer pipeline to files.
//!
//! ## Pipelines
//! Webcam to video
//! > gst-launch-1.0 -e v4l2src ! videoconvert ! autovideosink
//!
//! Webcam to JPEG
//! > gst-launch-1.0 -e v4l2src ! videoconvert ! jpegenc ! multifilesink location="./jpg_frame%09d.jpg"
//!
//! Webcam to raw image data
//! > gst-launch-1.0 -e v4l2src ! videoconvert ! video/x-raw,format=\(string\)RGB ! multifilesink location="./raw_frame%09d.data"
//!
//! RTSP to files
//! > gst-launch-1.0 -e rtspsrc location=rtsp://user:pw@10.0.0.1:554/channels/1 protocols=tcp ! rtph264depay ! video/x-h264 ! h264parse ! capsfilter ! avdec_h264 ! videoconvert ! video/x-raw,format=\(string\)RGB ! multifilesink location="./raw_frame%09d.data"
//!
//! ## Pipeline graph
//! ```bash
//! # Run pipeline generating `.dot` files
//! GST_DEBUG_DUMP_DOT_DIR=`pwd` gst-launch-1.0 -e v4l2src ! videoconvert ! decodebin ! jpegenc ! multifilesink location="./jpg_frame%09d.jpg"
//!
//! # Compile `.dot` files to a graph PNG
//! dot -T png 0.00.00.868970319-gst-launch.PAUSED_PLAYING.dot -o paused_playing.png
//! ```
//!
use byte_slice_cast::*;
use gstreamer as gst;
use gstreamer::element_error;
use gstreamer::prelude::*;
use gstreamer::MessageView;
use gstreamer_app as gst_app;
use image::RgbImage;
use signal_hook::{consts::SIGINT, iterator::Signals};
use simple_error::SimpleError;
use std::fs;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

type Error = Box<dyn std::error::Error>;

struct Frame {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

fn create_pipeline(width: u32, height: u32, tx: Sender<Frame>) -> Result<gst::Pipeline, Error> {
    gst::init()?;
    let pipeline_str = format!("v4l2src ! videoconvert ! video/x-raw,format=(string)RGB,width={},height={} ! appsink name=buffer_sink", width, height);

    let pipeline = gst::parse_launch(&pipeline_str)?;
    let pipeline = pipeline
        .dynamic_cast::<gst::Pipeline>()
        .expect("Should be dynamic pipeline");

    // Get appsink element from the pipeline
    let appsink = pipeline
        .by_name("buffer_sink")
        .expect("Should find element by name")
        .dynamic_cast::<gst_app::AppSink>()
        .expect("Should be appsink");

    // Set the format we expect on the sink
    appsink.set_caps(Some(&gst::Caps::builder("video/x-raw").build()));

    appsink.set_callbacks(
        gst_app::AppSinkCallbacks::builder()
            .new_sample(move |appsink| {
                // Pull sample from the buffer of appsink
                let sample = appsink.pull_sample().expect("Should get sample");
                let buffer = sample.buffer().ok_or_else(|| {
                    element_error!(
                        appsink,
                        gst::ResourceError::Failed,
                        ("Failed to get buffer from appsink")
                    );

                    gst::FlowError::Error
                })?;

                // Make buffer readable (if could be in RAM/GPU memory)
                let map = buffer.map_readable().map_err(|_| {
                    element_error!(
                        appsink,
                        gst::ResourceError::Failed,
                        ("Failed to map the buffer readable")
                    );

                    gst::FlowError::Error
                })?;

                // Interpret buffer as requested datatype
                let samples = map.as_slice_of::<u8>().map_err(|_| {
                    element_error!(
                        appsink,
                        gst::ResourceError::Failed,
                        ("Failed to interpret buffer as u8")
                    );

                    gst::FlowError::Error
                })?;

                println!("Got buffer of size {}", samples.len());

                let frame = Frame {
                    width,
                    height,
                    data: samples.to_vec(),
                };

                tx.send(frame).expect("Should be able to send frame");

                Ok(gst::FlowSuccess::Ok)
            })
            .build(),
    );

    Ok(pipeline)
}

fn run_pipeline(pipeline: gst::Pipeline) -> Result<(), Error> {
    pipeline.set_state(gst::State::Playing)?;

    let bus = pipeline.bus().expect("Pipeline should have a bus");

    for msg in bus.iter_timed(gst::ClockTime::NONE) {
        match msg.view() {
            MessageView::Eos(..) => break,
            MessageView::Error(err) => {
                pipeline.set_state(gst::State::Null)?;
                return Err(
                    SimpleError::new(&format!("Error in pipeline: {}", err.error())).into(),
                );
            }
            MessageView::StateChanged(change) => {
                if change.src().map(|s| s == pipeline).unwrap_or(false) {
                    println!(
                        "Pipeline changed state from {:?} to {:?}",
                        change.old(),
                        change.current()
                    );
                }
            }
            _ => (),
        }
    }

    pipeline.set_state(gst::State::Null)?;

    Ok(())
}

fn receive_and_save_frames(rx: Receiver<Frame>, out_dir: &str) -> Result<(), Error> {
    let mut counter = 0;
    loop {
        let frame = rx.recv()?;
        if let Some(image) = RgbImage::from_raw(frame.width, frame.height, frame.data) {
            let filename = format!("{}/frame_{:0>8}.jpg", out_dir, counter);
            image.save(&filename)?;
            println!("Saved frame to {}", filename);
            counter += 1;
        }
    }
}

fn main() -> Result<(), Error> {
    let mut signals = Signals::new(&[SIGINT])?;

    let out_dir = "out";

    fs::create_dir_all(out_dir)?;

    let signal_thread = thread::spawn(move || {
        for signal in signals.forever() {
            println!("Received signal {}, terminating", signal);
            std::process::exit(-1);
        }
    });

    let (tx, rx) = mpsc::channel();

    let saver_thread = thread::spawn(move || receive_and_save_frames(rx, out_dir).unwrap());

    let pipeline = create_pipeline(1280, 720, tx)?;
    let gstreamer_thread = thread::spawn(move || run_pipeline(pipeline).unwrap());

    println!("Streaming to {}", out_dir);

    loop {
        for thread_handle in [&signal_thread, &saver_thread, &gstreamer_thread] {
            if thread_handle.is_finished() {
                println!(
                    "Thread with ID {:?} exited unexpectedly",
                    thread_handle.thread().id()
                );
                std::process::exit(-1);
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}
