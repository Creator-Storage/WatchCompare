package network.creative.watchcompare.encoder

import android.app.Activity
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.media.AudioFormat
import android.media.MediaCodec
import android.media.MediaCodecInfo
import android.media.MediaExtractor
import android.media.MediaFormat
import android.media.MediaMuxer
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.io.File
import java.nio.ByteBuffer
import java.util.ArrayDeque
import kotlin.math.max
import kotlin.math.min

@InvokeArg
class BeginArgs {
    var outputPath: String = ""
    var width: Int = 1920
    var height: Int = 1080
    var fps: Int = 60
    var bitrate: Int = 18_000_000
    var frameCount: Long = 0
}

@InvokeArg
class PushFrameArgs {
    var path: String = ""
    var frameIndex: Long = 0
}

@InvokeArg
class FinishArgs {
    var soundtrackPath: String? = null
    var audioBitrate: Int = 192_000
}

@TauriPlugin
class WatchCompareEncoderPlugin(private val activity: Activity) : Plugin(activity) {
    private var session: VideoSession? = null

    @Synchronized
    @Command
    fun begin(invoke: Invoke) {
        try {
            session?.closeQuietly()
            val args = invoke.parseArgs(BeginArgs::class.java)
            require(args.outputPath.isNotBlank()) { "outputPath is required" }
            require(args.width > 0 && args.height > 0) { "invalid video dimensions" }
            require(args.width % 2 == 0 && args.height % 2 == 0) { "H.264 output dimensions must be even" }
            require(args.fps in 1..240) { "fps must be between 1 and 240" }
            require(args.frameCount > 0) { "frameCount must be positive" }
            session = VideoSession(args)
            invoke.resolve()
        } catch (error: Throwable) {
            session?.closeQuietly()
            session = null
            invoke.reject(error.message ?: error.toString())
        }
    }

    @Synchronized
    @Command
    fun pushFrame(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(PushFrameArgs::class.java)
            val active = session ?: error("encoder session has not been started")
            active.pushFrame(args.path, args.frameIndex)
            invoke.resolve()
        } catch (error: Throwable) {
            invoke.reject(error.message ?: error.toString())
        }
    }

    @Synchronized
    @Command
    fun finish(invoke: Invoke) {
        val active = session
        if (active == null) {
            invoke.reject("encoder session has not been started")
            return
        }
        try {
            val args = invoke.parseArgs(FinishArgs::class.java)
            val result = active.finish(args.soundtrackPath, args.audioBitrate)
            session = null
            val response = JSObject()
            response.put("path", result.path)
            response.put("videoCodec", result.videoCodec)
            response.put("audioCodec", result.audioCodec)
            invoke.resolve(response)
        } catch (error: Throwable) {
            active.closeQuietly()
            session = null
            invoke.reject(error.message ?: error.toString())
        }
    }

    @Synchronized
    @Command
    fun cancel(invoke: Invoke) {
        session?.closeQuietly()
        session = null
        invoke.resolve()
    }
}

private data class EncodeResult(
    val path: String,
    val videoCodec: String,
    val audioCodec: String?,
)

private class VideoSession(private val args: BeginArgs) {
    private val outputFile = File(args.outputPath)
    private val tempVideo = File(outputFile.parentFile ?: File("."), ".${outputFile.name}.watchcompare-video.mp4")
    private val encoder: MediaCodec
    private val muxer: MediaMuxer
    private val outputInfo = MediaCodec.BufferInfo()
    private var muxerStarted = false
    private var videoTrack = -1
    private var closed = false
    private var lastFrame = -1L

    init {
        outputFile.parentFile?.mkdirs()
        tempVideo.delete()
        outputFile.delete()

        val format = MediaFormat.createVideoFormat(MediaFormat.MIMETYPE_VIDEO_AVC, args.width, args.height)
        format.setInteger(MediaFormat.KEY_COLOR_FORMAT, MediaCodecInfo.CodecCapabilities.COLOR_FormatYUV420Flexible)
        format.setInteger(MediaFormat.KEY_BIT_RATE, max(1_000_000, args.bitrate))
        format.setInteger(MediaFormat.KEY_FRAME_RATE, args.fps)
        format.setInteger(MediaFormat.KEY_I_FRAME_INTERVAL, 1)
        if (android.os.Build.VERSION.SDK_INT >= 29) {
            format.setInteger(MediaFormat.KEY_MAX_B_FRAMES, 0)
        }

        encoder = MediaCodec.createEncoderByType(MediaFormat.MIMETYPE_VIDEO_AVC)
        encoder.configure(format, null, null, MediaCodec.CONFIGURE_FLAG_ENCODE)
        encoder.start()
        muxer = MediaMuxer(tempVideo.absolutePath, MediaMuxer.OutputFormat.MUXER_OUTPUT_MPEG_4)
    }

    fun pushFrame(path: String, frameIndex: Long) {
        check(!closed) { "encoder session is closed" }
        require(frameIndex in 0 until args.frameCount) { "frame index $frameIndex outside project range" }
        require(frameIndex > lastFrame) { "frames must be pushed in strictly increasing order" }
        val file = File(path)
        require(file.isFile) { "rendered frame does not exist: $path" }

        val decoded = BitmapFactory.decodeFile(file.absolutePath)
            ?: error("Android could not decode rendered frame: $path")
        val bitmap = ensureSize(decoded, args.width, args.height)
        try {
            queueBitmap(bitmap, frameIndex)
            drainEncoder(false)
            lastFrame = frameIndex
        } finally {
            if (bitmap !== decoded) bitmap.recycle()
            decoded.recycle()
        }
    }

    fun finish(soundtrackPath: String?, audioBitrate: Int): EncodeResult {
        check(!closed) { "encoder session is closed" }
        require(lastFrame >= 0) { "no frames were submitted" }
        queueEndOfStream()
        drainEncoder(true)
        releaseVideoEncoder()

        val soundtrack = soundtrackPath?.takeIf { it.isNotBlank() }?.let(::File)
        var audioCodec: String? = null
        if (soundtrack != null) {
            require(soundtrack.isFile) { "soundtrack does not exist: ${soundtrack.absolutePath}" }
            val prepared = prepareAacTrack(soundtrack, audioBitrate)
            audioCodec = MediaFormat.MIMETYPE_AUDIO_AAC
            try {
                muxVideoAndAudio(tempVideo, prepared.file, outputFile, durationUs())
            } finally {
                if (prepared.temporary) prepared.file.delete()
                tempVideo.delete()
            }
        } else {
            moveReplacing(tempVideo, outputFile)
        }
        closed = true
        return EncodeResult(outputFile.absolutePath, MediaFormat.MIMETYPE_VIDEO_AVC, audioCodec)
    }

    fun closeQuietly() {
        if (closed) return
        try { encoder.stop() } catch (_: Throwable) {}
        try { encoder.release() } catch (_: Throwable) {}
        try { if (muxerStarted) muxer.stop() } catch (_: Throwable) {}
        try { muxer.release() } catch (_: Throwable) {}
        tempVideo.delete()
        closed = true
    }

    private fun queueBitmap(bitmap: Bitmap, frameIndex: Long) {
        var inputIndex = encoder.dequeueInputBuffer(10_000)
        while (inputIndex < 0) {
            drainEncoder(false)
            inputIndex = encoder.dequeueInputBuffer(10_000)
        }

        val capacity = encoder.getInputBuffer(inputIndex)?.capacity()
            ?: (args.width * args.height * 3 / 2)
        val image = encoder.getInputImage(inputIndex)
            ?: error("device H.264 encoder does not expose flexible YUV input images")
        try {
            fillYuv420Image(bitmap, image.planes, args.width, args.height)
        } finally {
            image.close()
        }
        val ptsUs = frameIndex * 1_000_000L / args.fps.toLong()
        encoder.queueInputBuffer(inputIndex, 0, capacity, ptsUs, 0)
    }

    private fun queueEndOfStream() {
        var inputIndex = encoder.dequeueInputBuffer(10_000)
        while (inputIndex < 0) {
            drainEncoder(false)
            inputIndex = encoder.dequeueInputBuffer(10_000)
        }
        encoder.queueInputBuffer(
            inputIndex,
            0,
            0,
            args.frameCount * 1_000_000L / args.fps.toLong(),
            MediaCodec.BUFFER_FLAG_END_OF_STREAM,
        )
    }

    private fun drainEncoder(expectEos: Boolean) {
        var idlePasses = 0
        while (true) {
            val index = encoder.dequeueOutputBuffer(outputInfo, if (expectEos) 10_000 else 0)
            when {
                index == MediaCodec.INFO_TRY_AGAIN_LATER -> {
                    if (!expectEos || ++idlePasses > 500) return
                }
                index == MediaCodec.INFO_OUTPUT_FORMAT_CHANGED -> {
                    check(!muxerStarted) { "video encoder changed format twice" }
                    videoTrack = muxer.addTrack(encoder.outputFormat)
                    muxer.start()
                    muxerStarted = true
                }
                index >= 0 -> {
                    idlePasses = 0
                    val output = encoder.getOutputBuffer(index)
                        ?: error("encoder returned a null output buffer")
                    if ((outputInfo.flags and MediaCodec.BUFFER_FLAG_CODEC_CONFIG) != 0) {
                        outputInfo.size = 0
                    }
                    if (outputInfo.size > 0) {
                        check(muxerStarted && videoTrack >= 0) { "muxer was not started before encoded video data" }
                        output.position(outputInfo.offset)
                        output.limit(outputInfo.offset + outputInfo.size)
                        muxer.writeSampleData(videoTrack, output, outputInfo)
                    }
                    val eos = (outputInfo.flags and MediaCodec.BUFFER_FLAG_END_OF_STREAM) != 0
                    encoder.releaseOutputBuffer(index, false)
                    if (eos) return
                }
            }
        }
    }

    private fun releaseVideoEncoder() {
        if (closed) return
        encoder.stop()
        encoder.release()
        if (muxerStarted) muxer.stop()
        muxer.release()
    }

    private fun durationUs(): Long = args.frameCount * 1_000_000L / args.fps.toLong()
}

private data class PreparedAudio(val file: File, val temporary: Boolean)
private data class PcmChunk(val bytes: ByteArray, val ptsUs: Long, var offset: Int = 0)

private fun prepareAacTrack(source: File, bitrate: Int): PreparedAudio {
    val extractor = MediaExtractor()
    extractor.setDataSource(source.absolutePath)
    val track = findTrack(extractor, "audio/")
    if (track < 0) {
        extractor.release()
        error("soundtrack has no audio track")
    }
    val format = extractor.getTrackFormat(track)
    val mime = format.getString(MediaFormat.KEY_MIME) ?: ""
    extractor.release()
    if (mime == MediaFormat.MIMETYPE_AUDIO_AAC) {
        return PreparedAudio(source, false)
    }
    val temp = File(source.parentFile ?: File("."), ".${source.name}.watchcompare-aac-${System.nanoTime()}.mp4")
    transcodeAudioToAac(source, temp, bitrate)
    return PreparedAudio(temp, true)
}

private fun transcodeAudioToAac(source: File, output: File, bitrate: Int) {
    output.delete()
    val extractor = MediaExtractor()
    extractor.setDataSource(source.absolutePath)
    val sourceTrack = findTrack(extractor, "audio/")
    require(sourceTrack >= 0) { "soundtrack has no audio track" }
    extractor.selectTrack(sourceTrack)
    val inputFormat = extractor.getTrackFormat(sourceTrack)
    val inputMime = inputFormat.getString(MediaFormat.KEY_MIME) ?: error("soundtrack audio MIME missing")
    val sampleRate = inputFormat.getInteger(MediaFormat.KEY_SAMPLE_RATE)
    val channels = inputFormat.getInteger(MediaFormat.KEY_CHANNEL_COUNT)

    val decoder = MediaCodec.createDecoderByType(inputMime)
    decoder.configure(inputFormat, null, null, 0)
    decoder.start()

    val aacFormat = MediaFormat.createAudioFormat(MediaFormat.MIMETYPE_AUDIO_AAC, sampleRate, channels)
    aacFormat.setInteger(MediaFormat.KEY_AAC_PROFILE, MediaCodecInfo.CodecProfileLevel.AACObjectLC)
    aacFormat.setInteger(MediaFormat.KEY_BIT_RATE, max(64_000, bitrate))
    aacFormat.setInteger(MediaFormat.KEY_MAX_INPUT_SIZE, 256 * 1024)
    val encoder = MediaCodec.createEncoderByType(MediaFormat.MIMETYPE_AUDIO_AAC)
    encoder.configure(aacFormat, null, null, MediaCodec.CONFIGURE_FLAG_ENCODE)
    encoder.start()

    val muxer = MediaMuxer(output.absolutePath, MediaMuxer.OutputFormat.MUXER_OUTPUT_MPEG_4)
    val decoderInfo = MediaCodec.BufferInfo()
    val encoderInfo = MediaCodec.BufferInfo()
    val pending = ArrayDeque<PcmChunk>()
    var sourceDone = false
    var decoderDone = false
    var encoderInputDone = false
    var encoderDone = false
    var muxerStarted = false
    var audioTrack = -1

    try {
        while (!encoderDone) {
            if (!sourceDone) {
                val inputIndex = decoder.dequeueInputBuffer(0)
                if (inputIndex >= 0) {
                    val input = decoder.getInputBuffer(inputIndex) ?: error("decoder input buffer unavailable")
                    val size = extractor.readSampleData(input, 0)
                    if (size < 0) {
                        decoder.queueInputBuffer(inputIndex, 0, 0, 0, MediaCodec.BUFFER_FLAG_END_OF_STREAM)
                        sourceDone = true
                    } else {
                        decoder.queueInputBuffer(inputIndex, 0, size, extractor.sampleTime, extractor.sampleFlags)
                        extractor.advance()
                    }
                }
            }

            if (!decoderDone) {
                val outIndex = decoder.dequeueOutputBuffer(decoderInfo, 0)
                when {
                    outIndex == MediaCodec.INFO_OUTPUT_FORMAT_CHANGED -> Unit
                    outIndex >= 0 -> {
                        val out = decoder.getOutputBuffer(outIndex)
                        if (decoderInfo.size > 0 && out != null) {
                            out.position(decoderInfo.offset)
                            out.limit(decoderInfo.offset + decoderInfo.size)
                            val bytes = ByteArray(decoderInfo.size)
                            out.get(bytes)
                            pending.add(PcmChunk(bytes, decoderInfo.presentationTimeUs))
                        }
                        if ((decoderInfo.flags and MediaCodec.BUFFER_FLAG_END_OF_STREAM) != 0) decoderDone = true
                        decoder.releaseOutputBuffer(outIndex, false)
                    }
                }
            }

            if (pending.isNotEmpty()) {
                val inputIndex = encoder.dequeueInputBuffer(0)
                if (inputIndex >= 0) {
                    val input = encoder.getInputBuffer(inputIndex) ?: error("AAC encoder input unavailable")
                    val chunk = pending.first()
                    val bytesPerAudioFrame = max(1, channels * 2)
                    val writable = min(input.remaining(), chunk.bytes.size - chunk.offset)
                    input.put(chunk.bytes, chunk.offset, writable)
                    val frameOffset = chunk.offset / bytesPerAudioFrame
                    val pts = chunk.ptsUs + frameOffset * 1_000_000L / sampleRate.toLong()
                    encoder.queueInputBuffer(inputIndex, 0, writable, pts, 0)
                    chunk.offset += writable
                    if (chunk.offset >= chunk.bytes.size) pending.removeFirst()
                }
            } else if (decoderDone && !encoderInputDone) {
                val inputIndex = encoder.dequeueInputBuffer(0)
                if (inputIndex >= 0) {
                    encoder.queueInputBuffer(inputIndex, 0, 0, 0, MediaCodec.BUFFER_FLAG_END_OF_STREAM)
                    encoderInputDone = true
                }
            }

            val encodedIndex = encoder.dequeueOutputBuffer(encoderInfo, if (encoderInputDone) 10_000 else 0)
            when {
                encodedIndex == MediaCodec.INFO_OUTPUT_FORMAT_CHANGED -> {
                    audioTrack = muxer.addTrack(encoder.outputFormat)
                    muxer.start()
                    muxerStarted = true
                }
                encodedIndex >= 0 -> {
                    val encoded = encoder.getOutputBuffer(encodedIndex)
                    if ((encoderInfo.flags and MediaCodec.BUFFER_FLAG_CODEC_CONFIG) != 0) encoderInfo.size = 0
                    if (encoderInfo.size > 0 && encoded != null) {
                        check(muxerStarted) { "AAC muxer not started" }
                        encoded.position(encoderInfo.offset)
                        encoded.limit(encoderInfo.offset + encoderInfo.size)
                        muxer.writeSampleData(audioTrack, encoded, encoderInfo)
                    }
                    if ((encoderInfo.flags and MediaCodec.BUFFER_FLAG_END_OF_STREAM) != 0) encoderDone = true
                    encoder.releaseOutputBuffer(encodedIndex, false)
                }
            }
        }
    } finally {
        try { extractor.release() } catch (_: Throwable) {}
        try { decoder.stop() } catch (_: Throwable) {}
        try { decoder.release() } catch (_: Throwable) {}
        try { encoder.stop() } catch (_: Throwable) {}
        try { encoder.release() } catch (_: Throwable) {}
        try { if (muxerStarted) muxer.stop() } catch (_: Throwable) {}
        try { muxer.release() } catch (_: Throwable) {}
    }
}

private fun muxVideoAndAudio(video: File, audio: File, output: File, durationUs: Long) {
    output.delete()
    val videoExtractor = MediaExtractor()
    val audioExtractor = MediaExtractor()
    videoExtractor.setDataSource(video.absolutePath)
    audioExtractor.setDataSource(audio.absolutePath)
    val videoSourceTrack = findTrack(videoExtractor, "video/")
    val audioSourceTrack = findTrack(audioExtractor, "audio/")
    require(videoSourceTrack >= 0) { "encoded video track missing" }
    require(audioSourceTrack >= 0) { "prepared audio track missing" }
    videoExtractor.selectTrack(videoSourceTrack)
    audioExtractor.selectTrack(audioSourceTrack)

    val muxer = MediaMuxer(output.absolutePath, MediaMuxer.OutputFormat.MUXER_OUTPUT_MPEG_4)
    val videoTargetTrack = muxer.addTrack(videoExtractor.getTrackFormat(videoSourceTrack))
    val audioTargetTrack = muxer.addTrack(audioExtractor.getTrackFormat(audioSourceTrack))
    muxer.start()
    try {
        copyExtractorTrack(videoExtractor, muxer, videoTargetTrack, Long.MAX_VALUE)
        copyExtractorTrack(audioExtractor, muxer, audioTargetTrack, durationUs)
    } finally {
        try { muxer.stop() } finally { muxer.release() }
        videoExtractor.release()
        audioExtractor.release()
    }
}

private fun copyExtractorTrack(extractor: MediaExtractor, muxer: MediaMuxer, targetTrack: Int, stopAtUs: Long) {
    var capacity = 512 * 1024
    var buffer = ByteBuffer.allocateDirect(capacity)
    val info = MediaCodec.BufferInfo()
    while (true) {
        buffer.clear()
        var size = extractor.readSampleData(buffer, 0)
        if (size < 0) break
        if (size > capacity) {
            capacity = size * 2
            buffer = ByteBuffer.allocateDirect(capacity)
            continue
        }
        val pts = extractor.sampleTime
        if (pts < 0 || pts > stopAtUs) break
        info.set(0, size, pts, extractor.sampleFlags)
        buffer.position(0)
        buffer.limit(size)
        muxer.writeSampleData(targetTrack, buffer, info)
        extractor.advance()
    }
}

private fun findTrack(extractor: MediaExtractor, prefix: String): Int {
    for (index in 0 until extractor.trackCount) {
        val mime = extractor.getTrackFormat(index).getString(MediaFormat.KEY_MIME) ?: continue
        if (mime.startsWith(prefix)) return index
    }
    return -1
}

private fun ensureSize(bitmap: Bitmap, width: Int, height: Int): Bitmap {
    if (bitmap.width == width && bitmap.height == height) return bitmap
    return Bitmap.createScaledBitmap(bitmap, width, height, true)
}

private fun fillYuv420Image(bitmap: Bitmap, planes: Array<android.media.Image.Plane>, width: Int, height: Int) {
    require(planes.size >= 3) { "encoder YUV image does not expose three planes" }
    val pixels = IntArray(width * height)
    bitmap.getPixels(pixels, 0, width, 0, 0, width, height)
    val yPlane = planes[0]
    val uPlane = planes[1]
    val vPlane = planes[2]
    val yBuffer = yPlane.buffer
    val uBuffer = uPlane.buffer
    val vBuffer = vPlane.buffer

    for (y in 0 until height) {
        val yRow = y * yPlane.rowStride
        for (x in 0 until width) {
            val pixel = pixels[y * width + x]
            val r = (pixel shr 16) and 0xff
            val g = (pixel shr 8) and 0xff
            val b = pixel and 0xff
            val yy = (((66 * r + 129 * g + 25 * b + 128) shr 8) + 16).coerceIn(0, 255)
            yBuffer.put(yRow + x * yPlane.pixelStride, yy.toByte())
        }
    }

    for (y in 0 until height step 2) {
        val chromaY = y / 2
        val uRow = chromaY * uPlane.rowStride
        val vRow = chromaY * vPlane.rowStride
        for (x in 0 until width step 2) {
            var r = 0
            var g = 0
            var b = 0
            var count = 0
            for (dy in 0..1) {
                for (dx in 0..1) {
                    val sx = min(width - 1, x + dx)
                    val sy = min(height - 1, y + dy)
                    val pixel = pixels[sy * width + sx]
                    r += (pixel shr 16) and 0xff
                    g += (pixel shr 8) and 0xff
                    b += pixel and 0xff
                    count++
                }
            }
            r /= count
            g /= count
            b /= count
            val u = (((-38 * r - 74 * g + 112 * b + 128) shr 8) + 128).coerceIn(0, 255)
            val v = (((112 * r - 94 * g - 18 * b + 128) shr 8) + 128).coerceIn(0, 255)
            val chromaX = x / 2
            uBuffer.put(uRow + chromaX * uPlane.pixelStride, u.toByte())
            vBuffer.put(vRow + chromaX * vPlane.pixelStride, v.toByte())
        }
    }
}

private fun moveReplacing(source: File, target: File) {
    target.parentFile?.mkdirs()
    target.delete()
    if (!source.renameTo(target)) {
        source.copyTo(target, overwrite = true)
        source.delete()
    }
}
