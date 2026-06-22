# AI Interview Copilot

Desktop-приложение для помощи на технических интервью в реальном времени.

## Архитектура

```
client/   — Tauri + React (десктоп, скрытое окно)
server/   — FastAPI + WebSocket (локальный бэкенд)
```

## Запуск

### 1. Сервер
```bash
cd server
cp .env.example .env  # вставь OpenAI API ключ
pip install -r requirements.txt
python main.py
```

### 2. Клиент
```bash
cd client
npm install
npm run tauri dev
```

## Хоткей
- `Ctrl+Shift+H` — показать/скрыть окно
- Окно скрыто с Alt+Tab и панели задач

## MVP Features
- [x] WebSocket соединение с бэкендом
- [x] Генерация ответа через GPT
- [ ] Захват системного аудио
- [ ] Whisper распознавание речи
- [ ] Перевод вопроса
- [ ] Stealth mode (готово)
