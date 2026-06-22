from fastapi import FastAPI, WebSocket, WebSocketDisconnect, UploadFile, File
from fastapi.middleware.cors import CORSMiddleware
import json, os, asyncio, io
import numpy as np
from dotenv import load_dotenv
from faster_whisper import WhisperModel

load_dotenv()

app = FastAPI(title="AI Interview Copilot API")
app.add_middleware(CORSMiddleware, allow_origins=["*"], allow_methods=["*"], allow_headers=["*"])

# Init Whisper locally (tiny model = 75MB, base = 150MB, small = 500MB)
print("Loading Whisper model...")
model = WhisperModel("base", device="cpu", compute_type="int8")
print("Whisper loaded")

# DeepSeek client
from openai import OpenAI
deepseek = OpenAI(
    api_key=os.getenv("DEEPSEEK_API_KEY", "sk-dummy"),
    base_url="https://api.deepseek.com"
)

SYSTEM_PROMPT = """Ты AI-ассистент для технических интервью.
Получаешь текст вопроса от интервьюера. Ответь кратко (3-5 предложений), профессионально.
В конце добавь 2-3 ключевых тезиса для ответа своими словами.
Адаптируй под уровень и стек кандидата."""

@app.get("/health")
async def health():
    return {"status": "ok"}

@app.post("/transcribe")
async def transcribe(file: UploadFile = File(...)):
    """Receive audio WAV bytes, return transcribed text"""
    audio_bytes = await file.read()
    
    # Save to temp file (faster-whisper needs file path or numpy)
    tmp_path = "/tmp/_capture.wav"
    with open(tmp_path, "wb") as f:
        f.write(audio_bytes)
    
    segments, info = model.transcribe(tmp_path, beam_size=3, language="en")
    text = " ".join(seg.text for seg in segments)
    
    return {"text": text.strip()}

async def generate_answer(question: str, history: list) -> str:
    messages = [{"role": "system", "content": SYSTEM_PROMPT}]
    for h in history[-10:]:
        messages.append({"role": "user", "content": h["q"]})
        messages.append({"role": "assistant", "content": h["a"]})
    messages.append({"role": "user", "content": f"Вопрос: {question}"})
    
    resp = deepseek.chat.completions.create(
        model="deepseek-chat",
        messages=messages,
        max_tokens=500
    )
    return resp.choices[0].message.content

@app.websocket("/ws")
async def websocket_endpoint(ws: WebSocket):
    await ws.accept()
    history = []
    
    while True:
        try:
            data = await asyncio.wait_for(ws.receive_text(), timeout=120)
            msg = json.loads(data)
            
            if msg["type"] == "transcript":
                text = msg["text"]
                answer = await generate_answer(text, history)
                history.append({"q": text, "a": answer})
                await ws.send_json({
                    "type": "answer",
                    "question": text,
                    "answer": answer
                })
                
        except asyncio.TimeoutError:
            await ws.send_json({"type": "ping"})
        except WebSocketDisconnect:
            break
        except Exception as e:
            await ws.send_json({"type": "error", "message": str(e)})

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="127.0.0.1", port=3457)
