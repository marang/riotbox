"""Pure original-source WAV decode; file admission and graph analysis are separate."""

import io
import wave
from dataclasses import dataclass


_SUPPORTED_SAMPLE_WIDTHS = {1, 2, 3, 4}


@dataclass(frozen=True)
class DecodedSourceWave:
    sample_rate: int
    channel_count: int
    frame_count: int
    samples: list[float]


def decode_source_wave(content: bytes) -> DecodedSourceWave:
    data_size = _validate_container(content)
    try:
        with wave.open(io.BytesIO(content), "rb") as wav_file:
            if wav_file.getcomptype() != "NONE":
                raise ValueError(f"unsupported WAV compression: {wav_file.getcomptype()}")
            sample_rate = wav_file.getframerate()
            channel_count = wav_file.getnchannels()
            frame_count = wav_file.getnframes()
            sample_width = wav_file.getsampwidth()
            if sample_rate <= 0 or channel_count <= 0:
                raise ValueError("source WAV rate and channel count must be positive")
            if sample_width not in _SUPPORTED_SAMPLE_WIDTHS:
                raise ValueError(f"unsupported WAV sample width: {sample_width}")
            frame_width = channel_count * sample_width
            if data_size % frame_width:
                raise ValueError("source WAV data does not contain whole PCM frames")
            if frame_count * frame_width != data_size:
                raise ValueError("source WAV declared frame count does not match data size")
            frames = wav_file.readframes(frame_count)
    except (EOFError, wave.Error, RuntimeError) as error:
        # Only parser operations live inside this guard, not graph analysis.
        detail = str(error) or "truncated WAV header or PCM payload"
        raise ValueError(f"invalid source WAV: {detail}") from error

    if len(frames) != data_size:
        raise ValueError("source WAV decoded byte count does not match declared data size")
    samples = _decode_pcm_samples(frames, channel_count, sample_width)
    return DecodedSourceWave(sample_rate, channel_count, frame_count, samples)


def _validate_container(content: bytes) -> int:
    """Return the unique data size without retaining or copying chunk payloads."""
    if len(content) < 12 or content[:4] != b"RIFF" or content[8:12] != b"WAVE":
        raise ValueError("source WAV has no complete RIFF/WAVE header")
    container_end = 8 + int.from_bytes(content[4:8], "little")
    if container_end != len(content):
        raise ValueError("source WAV RIFF extent does not match captured byte count")

    cursor = 12
    format_seen = False
    data_size = None
    while cursor < container_end:
        if cursor + 8 > container_end:
            raise ValueError("source WAV has an incomplete chunk header")
        identifier = content[cursor:cursor + 4]
        size = int.from_bytes(content[cursor + 4:cursor + 8], "little")
        payload_end = cursor + 8 + size
        if payload_end > container_end:
            raise ValueError("source WAV chunk extends past the container end")
        if identifier == b"fmt ":
            if format_seen:
                raise ValueError("source WAV has duplicate fmt chunks")
            format_seen = True
        elif identifier == b"data":
            if not format_seen:
                raise ValueError("source WAV data precedes fmt")
            if data_size is not None:
                raise ValueError("source WAV has duplicate data chunks")
            data_size = size
        cursor = payload_end
        if size % 2:
            # Python's writer historically omits the last odd data pad. Keep
            # only that terminal-data exception; interior chunks require it.
            if cursor == container_end and identifier == b"data":
                break
            if cursor == container_end:
                raise ValueError("source WAV metadata chunk is missing its pad byte")
            cursor += 1
    if not format_seen or data_size is None:
        raise ValueError("source WAV is missing fmt or data")
    return data_size


def _decode_pcm_samples(frames: bytes, channel_count: int, sample_width: int) -> list[float]:
    if sample_width not in _SUPPORTED_SAMPLE_WIDTHS:
        raise ValueError(f"unsupported PCM sample width: {sample_width}")

    scale = float((1 << ((sample_width * 8) - 1)) - 1)
    frame_width = channel_count * sample_width
    sample_values = []

    for frame_start in range(0, len(frames), frame_width):
        frame = frames[frame_start : frame_start + frame_width]
        if len(frame) < frame_width:
            raise ValueError("source WAV PCM conversion encountered a partial frame")

        channel_sum = 0.0
        for channel_index in range(channel_count):
            start = channel_index * sample_width
            sample_bytes = frame[start : start + sample_width]
            if sample_width == 1:
                value = sample_bytes[0] - 128
                scale_value = 127.0
            else:
                value = int.from_bytes(sample_bytes, byteorder="little", signed=True)
                scale_value = scale
            channel_sum += float(value) / scale_value

        sample_values.append(channel_sum / channel_count)

    return sample_values
