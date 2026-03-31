# HealthBot — AI-Powered Patient Education System

An agentic AI chatbot that helps patients learn about health topics through personalized summaries and comprehension quizzes. Built with **LangGraph** and **Ollama**, it runs entirely locally — no paid API keys required.

## Features

- **Web-grounded answers** — runs two focused DuckDuckGo searches per topic (overview/symptoms and treatment/prevention) and feeds all results to the LLM
- **Patient-friendly summaries** — LLM condenses search results into plain-language explanations
- **Comprehension quizzes** — generates and grades a short-answer question based solely on the presented material
- **Multi-topic sessions** — loop through as many health topics as you like in a single session
- **Fully local** — runs on your machine via Ollama; no data sent to external AI APIs
- **Built-in chat UI** — interactive ipywidgets panel with a scrollable message area and a persistent input bar; no page scrolling required

## Architecture

The chatbot is implemented as a [LangGraph](https://github.com/langchain-ai/langgraph) state machine:

```
START → get_topic → search_topic → summarize_topic → present_summary
      → prompt_ready_for_quiz → generate_quiz → present_quiz
      → collect_answer → evaluate_answer → present_result
      → ask_continue ──► (yes) → get_topic
                     └─► (no)  → END
```

## Prerequisites

| Requirement | Notes |
|---|---|
| Python 3.10+ | Tested with 3.12 |
| [Ollama](https://ollama.com) | Must be installed and running locally |
| A pulled Ollama model | Default: `llama3.1:8b` |

## Setup

1. **Install Ollama** from [ollama.com](https://ollama.com) and pull a model:
   ```bash
   ollama pull llama3.1:8b
   ```

2. **Create and activate a virtual environment:**
   ```bash
   python -m venv .venv
   source .venv/bin/activate   # Windows: .venv\Scripts\activate
   ```

3. **Install Python dependencies:**
   ```bash
   pip install -r requirements.txt
   ```

4. **Start Ollama** (if not already running):
   ```bash
   ollama serve
   ```

## Usage

Open `medical-chatbot.ipynb` in **VS Code** (recommended) or Jupyter and run all cells in order:

```bash
# Jupyter alternative
jupyter notebook medical-chatbot.ipynb
```

The last cell launches the interactive HealthBot session. A chat panel appears directly in the cell output with a scrollable message area and an input bar pinned to the bottom. Type your responses and press **Enter** or click **Submit** — no page scrolling is needed.

> **Note:** The graph runs in a background thread so the kernel stays idle between responses. VS Code will dispatch button/Enter events only when the kernel is not busy, which is why the last cell returns immediately after starting the thread.

### Changing the model

Edit the `OLLAMA_MODEL` variable in the first cell:

```python
OLLAMA_MODEL = "mistral"   # or "gemma3", "phi3", "llama3.1:8b", etc.
```

## Project Structure

```
.
├── medical-chatbot.ipynb   # Main notebook — all code and session logic
├── requirements.txt        # Python dependencies
├── .gitignore
└── README.md
```

## Dependencies

| Package | Version | Purpose |
|---|---|---|
| `langchain-ollama` | >=0.1.0,<0.2.0 | Ollama integration (includes `langchain-core`) |
| `langgraph` | 0.2.19 | Agentic state machine |
| `ddgs` | >=1.0.0,<2.0.0 | DuckDuckGo web search (no API key; renamed from `duckduckgo-search`) |
| `ipywidgets` | ≥ 8.0 | Interactive chat UI inside the notebook |

## Disclaimer

HealthBot is an **educational tool only**. The information it provides is sourced from public websites and summarized by an AI model. It is **not a substitute for professional medical advice, diagnosis, or treatment**. Always consult a qualified healthcare provider with any questions about a medical condition.
