import { useState, useEffect, useRef } from "react";

type HistoryItem = {
  question: string;
  answer: string;
};

declare global {
  interface Window {
    __TAURI__?: {
      invoke: (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;
    };
  }
}

function App() {
  const [connected, setConnected] = useState(false);
  const [recording, setRecording] = useState(false);
  const [transcribing, setTranscribing] = useState(false);
  const [question, setQuestion] = useState("");
  const [answer, setAnswer] = useState("");
  const [history, setHistory] = useState<HistoryItem[]>([]);
  const [status, setStatus] = useState("Ожидание...");
  const wsRef = useRef<WebSocket | null>(null);

  const invoke = window.__TAURI__?.invoke;

  useEffect(() => {
    const ws = new WebSocket("ws://127.0.0.1:3457/ws");
    wsRef.current = ws;

    ws.onopen = () => { setConnected(true); setStatus("Готово"); };
    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      if (data.type === "answer") {
        setQuestion(data.question);
        setAnswer(data.answer);
        setHistory((prev) => [{ question: data.question, answer: data.answer }, ...prev]);
        setStatus("Готово");
      } else if (data.type === "error") {
        setStatus(`Ошибка: ${data.message}`);
      }
    };
    ws.onclose = () => { setConnected(false); setStatus("Отключено"); };
    return () => ws.close();
  }, []);

  const toggleCapture = async () => {
    if (!invoke) return;
    if (recording) {
      setTranscribing(true);
      setStatus("🔊 Распознавание...");
      try {
        const text = await invoke("stop_and_send") as string;
        if (text && wsRef.current) {
          wsRef.current.send(JSON.stringify({ type: "transcript", text }));
          setStatus("🤔 Генерация...");
        }
      } catch (e) {
        setStatus(`Ошибка: ${e}`);
      }
      setRecording(false);
      setTranscribing(false);
    } else {
      await invoke("start_capture");
      setRecording(true);
      setStatus("🎙 Запись...");
    }
  };

  const [inputText, setInputText] = useState("");
  const sendQuestion = () => {
    if (!inputText.trim() || !wsRef.current) return;
    wsRef.current.send(JSON.stringify({ type: "transcript", text: inputText }));
    setStatus("🤔 Генерация...");
    setInputText("");
  };

  return (
    <div className="app">
      <header>
        <h1>🎙 Copilot</h1>
        <span className={`badge ${connected ? "online" : "offline"}`}>{status}</span>
      </header>
      <main>
        <div className="controls">
          <button
            className={`btn ${recording ? "btn-stop" : "btn-rec"}`}
            onClick={toggleCapture}
            disabled={!invoke || transcribing}
          >
            {recording ? "⏹ Стоп" : transcribing ? "⏳..." : "🎙 Запись"}
          </button>
        </div>
        <div className="card question-card">
          <div className="label">❓ Вопрос</div>
          <div className="content">{question || "Ожидание..."}</div>
        </div>
        <div className="card answer-card">
          <div className="label">💡 Ответ</div>
          <div className="content">{answer || "..."}</div>
        </div>
        <div className="input-row">
          <input value={inputText} onChange={(e) => setInputText(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && sendQuestion()}
            placeholder="Тест: введи вопрос" />
          <button onClick={sendQuestion}>→</button>
        </div>
        {history.length > 0 && (
          <div className="history">
            <div className="label">📋 История</div>
            {history.slice(0, 10).map((item, i) => (
              <div key={i} className="history-item">
                <div className="q">Q: {item.question}</div>
                <div className="a">{item.answer.slice(0, 150)}...</div>
              </div>
            ))}
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
