<script setup lang="ts">
import { ref } from "vue";
import { startRecording, stopRecording } from "tauri-plugin-mic-recorder-api";
import { invoke } from "@tauri-apps/api/core";

const transcribedText = ref("");
const recording = ref(false);
const isTranscribing = ref(false);

async function start() {
  recording.value = true;
  await startRecording();
}

async function stop() {
  recording.value = false;
  const filePath = await stopRecording();
  await transcribe(filePath);
}

async function transcribe(filePath: string) {
  isTranscribing.value = true;
  transcribedText.value = "Transcribing, please wait...";
  try {
    const result = await invoke<string>("transcribe_rs", { path: filePath });
    transcribedText.value = result;
  } catch (error) {
    console.error("Error during transcription:", error);
    transcribedText.value = `Error: ${error}`;
  } finally {
    isTranscribing.value = false;
  }
}

function copyText() {
  if (transcribedText.value) {
    navigator.clipboard.writeText(transcribedText.value);
    alert("Text copied to clipboard!");
  }
}
</script>

<template>
  <main class="container">
    <h1>Whisper Voice-to-Text</h1>
    <div class="row">
      <button @click="start" :disabled="recording || isTranscribing">
        Record
      </button>
      <button @click="stop" :disabled="!recording || isTranscribing">
        Stop
      </button>
    </div>
    <div class="text-container">
      <textarea
        v-model="transcribedText"
        readonly
        placeholder="Transcribed text will appear here..."
      ></textarea>
      <button
        @click="copyText"
        :disabled="!transcribedText || isTranscribing"
        class="copy-button"
      >
        Copy
      </button>
    </div>
    <div v-if="isTranscribing" class="row">
      <p>Processing...</p>
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