use crate::audio::sounds::char_to_frequency;
use crate::error::Result;

/// 采样率 (Hz)
const SAMPLE_RATE: u32 = 8000;

/// 单个音节时长 (秒)
const TONE_DURATION: f64 = 0.4;

/// 静音间隔 (秒)
const SILENCE_DURATION: f64 = 0.15;

/// 生成正弦波音频数据
fn generate_tone(frequency: f64, duration: f64, amplitude: f64) -> Vec<i16> {
    let num_samples = (SAMPLE_RATE as f64 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f64 / SAMPLE_RATE as f64;
        let sample = (amplitude * (2.0 * std::f64::consts::PI * frequency * t).sin()) as i16;
        samples.push(sample);
    }

    // 应用衰减包络
    if num_samples > 0 {
        let decay_samples = (num_samples as f64 * 0.1) as usize;
        let start = num_samples.saturating_sub(decay_samples);
        for (offset, sample) in samples[start..].iter_mut().enumerate() {
            let i = start + offset;
            let factor = (num_samples - i) as f64 / decay_samples as f64;
            *sample = (*sample as f64 * factor) as i16;
        }
    }

    samples
}

/// 生成静音
fn generate_silence(duration: f64) -> Vec<i16> {
    let num_samples = (SAMPLE_RATE as f64 * duration) as usize;
    vec![0i16; num_samples]
}

/// 为验证码字符串生成 WAV 音频数据
///
/// 返回完整的 WAV 文件字节 (16-bit PCM, mono, 8kHz)
pub fn generate_captcha_wav(content: &str) -> Result<Vec<u8>> {
    let amplitude = 0.5;
    let mut all_samples: Vec<i16> = Vec::new();

    let chars: Vec<char> = content.chars().collect();
    for (i, ch) in chars.iter().enumerate() {
        let freq = char_to_frequency(*ch);
        let mut tone = generate_tone(freq, TONE_DURATION, amplitude);
        all_samples.append(&mut tone);

        if i < chars.len() - 1 {
            let mut silence = generate_silence(SILENCE_DURATION);
            all_samples.append(&mut silence);
        }
    }

    build_wav(&all_samples, SAMPLE_RATE)
}

/// 构建 WAV 文件字节
fn build_wav(samples: &[i16], sample_rate: u32) -> Result<Vec<u8>> {
    let num_channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * num_channels as u32 * bits_per_sample as u32 / 8;
    let block_align = num_channels * bits_per_sample / 8;
    let data_size = samples.len() as u32 * 2;
    let file_size = 36 + data_size;

    let mut buffer = Vec::with_capacity(44 + samples.len() * 2);

    // RIFF header
    buffer.extend_from_slice(b"RIFF");
    buffer.extend_from_slice(&file_size.to_le_bytes());
    buffer.extend_from_slice(b"WAVE");

    // fmt subchunk
    buffer.extend_from_slice(b"fmt ");
    buffer.extend_from_slice(&16u32.to_le_bytes());
    buffer.extend_from_slice(&1u16.to_le_bytes()); // PCM
    buffer.extend_from_slice(&num_channels.to_le_bytes());
    buffer.extend_from_slice(&sample_rate.to_le_bytes());
    buffer.extend_from_slice(&byte_rate.to_le_bytes());
    buffer.extend_from_slice(&block_align.to_le_bytes());
    buffer.extend_from_slice(&bits_per_sample.to_le_bytes());

    // data subchunk
    buffer.extend_from_slice(b"data");
    buffer.extend_from_slice(&data_size.to_le_bytes());

    for sample in samples {
        buffer.extend_from_slice(&sample.to_le_bytes());
    }

    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 从 WAV 字节中读取小端 u16/u32 字段
    fn le_u16(b: &[u8], off: usize) -> u16 {
        u16::from_le_bytes(b[off..off + 2].try_into().unwrap())
    }

    fn le_u32(b: &[u8], off: usize) -> u32 {
        u32::from_le_bytes(b[off..off + 4].try_into().unwrap())
    }

    #[test]
    fn test_generate_captcha_wav_header_fields() {
        let wav = generate_captcha_wav("1234").unwrap();

        // RIFF 头
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");

        // fmt 子块
        assert_eq!(&wav[12..16], b"fmt ");
        assert_eq!(le_u32(&wav, 16), 16); // fmt 块大小
        assert_eq!(le_u16(&wav, 20), 1); // PCM
        assert_eq!(le_u16(&wav, 22), 1); // 单声道
        assert_eq!(le_u32(&wav, 24), SAMPLE_RATE); // 采样率 8000
        assert_eq!(le_u32(&wav, 28), SAMPLE_RATE * 2); // byte_rate = 16000
        assert_eq!(le_u16(&wav, 32), 2); // block_align
        assert_eq!(le_u16(&wav, 34), 16); // 位深

        // data 子块与总长一致性
        assert_eq!(&wav[36..40], b"data");
        let data_size = le_u32(&wav, 40) as usize;
        assert_eq!(wav.len(), 44 + data_size);
        assert_eq!(le_u32(&wav, 4), (36 + data_size) as u32);

        // "1234": 4 个音 * 3200 样本 + 3 段静音 * 1200 样本
        let expected_samples = 4 * 3200 + 3 * 1200;
        assert_eq!(data_size, expected_samples * 2);
    }

    #[test]
    fn test_empty_content_produces_valid_empty_wav() {
        let wav = generate_captcha_wav("").unwrap();
        assert_eq!(le_u32(&wav, 40), 0);
        assert_eq!(wav.len(), 44);
    }
}
