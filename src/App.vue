<script setup lang="ts">
import { ref } from 'vue';
import { readBinaryFile } from '@tauri-apps/api/fs';
import { AutoTokenizer, AutoModelForSpeechSeq2Seq } from '@xenova/transformers';

const transcribedText = ref('');
const recording = ref(false);
let mediaRecorder: MediaRecorder | null = null;
let audioChunks: Blob[] = [];

async function getMicrophonePermission() {
  try {
    const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
    return stream;
  } catch (error) {
    console.error('Error getting microphone permission:', error);
    alert('Microphone permission is required to record audio.');
    return null;
  }
}

async function startRecording() {
  const stream = await getMicrophonePermission();
  if (stream) {
    recording.value = true;
    mediaRecorder = new MediaRecorder(stream);
    mediaRecorder.ondataavailable = (event) => {
      audioChunks.push(event.data);
    };
    mediaRecorder.onstop = async () => {
      const audioBlob = new Blob(audioChunks, { type: 'audio/wav' });
      audioChunks = [];
      await transcribe(audioBlob);
      recording.value = false;
    };
    mediaRecorder.start();
  }
}

function stopRecording() {
  if (mediaRecorder) {
    mediaRecorder.stop();
  }
}

async function transcribe(audioBlob: Blob) {
  const reader = new FileReader();
  reader.onload = async (event) => {
    const audioData = event.target?.result;
    if (audioData) {
      try {
        const modelPath = 'src/assets/models/whisper-tiny.en/';
        const config = JSON.parse(new TextDecoder().decode(await readBinaryFile(modelPath + 'config.json')));
        const tokenizer = await AutoTokenizer.from_pretrained(null, {
          tokenizer_file: new TextDecoder().decode(await readBinaryFile(modelPath + 'tokenizer.json')),
          ...config
        });
        const model = await AutoModelForSpeechSeq2Seq.from_pretrained(null, {
          model_file: await readBinaryFile(modelPath + 'model.onnx'),
          ...config
        });

        const audio = new Float32Array(await new AudioContext().decodeAudioData(audioData as ArrayBuffer));
        const input_features = tokenizer(audio, { return_tensors: "pt" }).input_features;
        const forced_decoder_ids = tokenizer.get_decoder_prompt_ids({ language: "english", task: "transcribe" });
        const predicted_ids = await model.generate(input_features, { forced_decoder_ids });
        const transcription = tokenizer.batch_decode(predicted_ids, { skip_special_tokens: true })[0];

        transcribedText.value = transcription;
      } catch (error) {
        console.error('Error during transcription:', error);
      }
    }
  };
  reader.readAsArrayBuffer(audioBlob);
}

function copyText() {
  if (transcribedText.value) {
    navigator.clipboard.writeText(transcribedText.value);
    alert('Text copied to clipboard!');
  }
}
</script>

<template>
  <main class="container">
    <h1>Whisper Voice-to-Text</h1>
    <div class="row">
      <button @click="startRecording" :disabled="recording">Record</button>
      <button @click="stopRecording" :disabled="!recording">Stop</button>
    </div>
    <div class="text-container">
      <textarea v-model="transcribedText" readonly placeholder="Transcribed text will appear here..."></textarea>
      <button @click="copyText" class="copy-button">Copy</button>
    </div>
  </main>
</template>

<style scoped>
.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

.row {
  display: flex;
  justify-content: center;
  margin-bottom: 1rem;
}

button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
  cursor: pointer;
  margin: 0 0.5rem;
}

button:hover {
  border-color: #396cd8;
}

button:disabled {
  background-color: #e8e8e8;
  cursor: not-allowed;
}

.text-container {
  position: relative;
  width: 80%;
  margin: 0 auto;
}

textarea {
  width: 100%;
  height: 200px;
  border-radius: 8px;
  border: 1px solid #ccc;
  padding: 0.6em;
  font-size: 1em;
  font-family: inherit;
  resize: none;
}

.copy-button {
  position: absolute;
  top: 10px;
  right: 10px;
  padding: 0.3em 0.6em;
  font-size: 0.8em;
}
</style>