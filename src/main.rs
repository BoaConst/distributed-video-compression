use ffmpeg_next as ffmpeg;

fn main() {
    // Open input video file
    let mut input_context = ffmpeg::format::input(&"input.mp4").unwrap();

    // Find video stream in input file
    let video_stream_index = input_context
        .streams()
        .best(ffmpeg::media::Type::Video)
        .unwrap()
        .index();

    // Get video stream parameters
    let video_stream = input_context.streams().get(video_stream_index).unwrap();
    let video_time_base = video_stream.time_base();
    let video_duration = video_stream.duration();
    let video_frame_rate = video_stream.avg_frame_rate();
    let video_codec_parameters = video_stream.codecpar();

    // Open output video file
    let mut output_context = ffmpeg::format::output(&"output.mp4").unwrap();

    // Add video stream to output file
    let mut output_video_stream = output_context
        .add_stream(ffmpeg::codec::id::AV_CODEC_ID_H264)
        .unwrap();

    // Copy codec parameters from input to output stream
    output_video_stream.set_parameters(video_codec_parameters);

    // Open video encoder
    let mut video_encoder = output_video_stream.codec().encoder().video().unwrap();

    // Set encoding parameters
    video_encoder.set_time_base(video_time_base);
    video_encoder.set_frame_rate(video_frame_rate);
    video_encoder.set_width(video_codec_parameters.width());
    video_encoder.set_height(video_codec_parameters.height());
    video_encoder.set_format(ffmpeg::format::Pixel::YUV420P);

    // Open output video file for writing
    output_context
        .write_header()
        .expect("Error writing header");

    // Loop over input video frames, encode each frame, and write to output file
    let mut packet = ffmpeg::packet::Packet::empty();
    while let Ok(Some(mut input_packet)) = input_context.read_packet() {
        if input_packet.stream_index() == video_stream_index {
            // Decode input packet into a frame
            let mut input_frame = ffmpeg::frame::Video::empty();
            input_packet.rescale_ts(video_time_base, video_stream.time_base());
            input_packet.decode(&mut input_frame, &video_stream.codec().decoder().unwrap()).unwrap();

            // Encode frame using video encoder
            let mut output_packet = ffmpeg::packet::Packet::empty();
            video_encoder.encode(&input_frame, &mut output_packet).unwrap();

            // Write compressed frame to output file
            if output_packet.size() > 0 {
                output_packet.rescale_ts(video_encoder.time_base(), output_video_stream.time_base());
                output_packet.set_stream_index(output_video_stream.index());
                output_context.write_packet(&mut output_packet).unwrap();
            }
        }
    }

    // Flush video encoder
    video_encoder.flush(&mut packet).unwrap();
    if packet.size() > 0 {
        packet.rescale_ts(video_encoder.time_base(), output_video_stream.time_base());
        packet.set_stream_index(output_video_stream.index());
        output_context.write_packet(&mut packet).unwrap();
    }

    // Close input and output files
    input_context.close_input().unwrap();
    output_context.close_output().unwrap();
}
