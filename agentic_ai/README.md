# Multi-Agent ChatBot — AI-Powered Learning Assistant

An agentic AI system that routes user questions to specialised chatbot agents — each an expert in its own domain. Built with **LangGraph** and **Ollama**, it runs entirely locally — no paid API keys required.

## Agents

| Agent | Domain |
|---|---|
| 🏥 **HealthBot** | Medical conditions, symptoms, treatments, wellness |
| 💻 **QualityBot** | Software quality, testing, QA, standards, best practices |

## Features

- **LLM-based routing** — an LLM classifier automatically selects the right specialist agent for each topic
- **Web-grounded answers** — each agent runs two focused DuckDuckGo searches and feeds all results to the LLM
- **Domain-specific summaries** — the LLM condenses search results into clear, accessible explanations
- **Comprehension quizzes** — generates and grades a short-answer question based solely on the presented material
- **Multi-topic sessions** — loop through as many topics as you like, switching agents as needed
- **Fully local** — runs on your machine via Ollama; no data sent to external AI APIs
- **Single chat UI** — one interactive ipywidgets panel with a scrollable message area; agents are identified by name in the chat

## Architecture

The system is implemented as a [LangGraph](https://github.com/langchain-ai/langgraph) state machine with an LLM-based router:

```
START → get_topic → classify_topic ─┬─► search_health    → summarize_health    ─┐
                                    └─► search_sw_quality → summarize_sw_quality ─┤
                                                                                  ▼
                                                                          present_summary
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

Open `multi-agent-chatbot.ipynb` in **VS Code** (recommended) or Jupyter and run all cells in order:

```bash
# Jupyter alternative
jupyter notebook multi-agent-chatbot.ipynb
```

The last cell launches the interactive Multi-Agent ChatBot session. A chat panel appears directly in the cell output with a scrollable message area and an input bar pinned to the bottom. Type any topic — the router will automatically select the right specialist agent. Press **Enter** or click **Submit** to interact.

> **Note:** The graph runs in a background thread so the kernel stays idle between responses. VS Code will dispatch button/Enter events only when the kernel is not busy, which is why the last cell returns immediately after starting the thread.

### Changing the model

Edit the `OLLAMA_MODEL` variable in the first cell:

```python
OLLAMA_MODEL = "mistral"   # or "gemma3", "phi3", "llama3.1:8b", etc.
```

## Project Structure

```
.
├── multi-agent-chatbot.ipynb   # Multi-Agent ChatBot — router + specialist agents
├── medical-chatbot..ipynb      # Standalone HealthBot — single-agent patient education
├── requirements.txt            # Python dependencies
├── .gitignore
└── README.md
```

## Dependencies

| Package | Version | Purpose |
|---|---|---|
| `langchain-ollama` | >=0.1.0,<0.2.0 | Ollama integration (includes `langchain-core`) |
| `langgraph` | 1.0.10rc1 | Agentic state machine |
| `ddgs` | >=1.0.0,<2.0.0 | DuckDuckGo web search (no API key; renamed from `duckduckgo-search`) |
| `ipywidgets` | ≥ 8.0 | Interactive chat UI inside the notebook |

## Notebooks

### `multi-agent-chatbot.ipynb` — Multi-Agent ChatBot

The primary notebook described above. Routes topics to specialist agents (HealthBot, QualityBot) via an LLM classifier.

### `medical-chatbot..ipynb` — Standalone HealthBot

A focused, single-agent version of HealthBot aimed at patient education. It shares the same LangGraph + Ollama stack but operates independently — no routing layer. Use this if you only need the medical domain:

1. Enter a health topic or condition
2. Receive a web-grounded, patient-friendly summary (2 DuckDuckGo searches)
3. Answer a short comprehension quiz and receive a graded result (A–F)
4. Continue with a new topic or end the session

Open `medical-chatbot..ipynb` and run all cells; the chat panel appears in the last cell output.

---

## Adding More Agents

To add a new specialist agent:

1. Add an entry to `AGENT_LABELS` (e.g. `"devops": "🔧 DevOpsBot"`)
2. Create `search_<domain>()` and `summarize_<domain>()` node functions
3. Register them as nodes in `build_chatbot()` and wire the edges
4. Add the new domain to the classifier's system prompt in `classify_topic()`

## Disclaimer

This chatbot is an **educational tool only**. All information is sourced from public websites and summarized by an AI model. The HealthBot agent is **not a substitute for professional medical advice, diagnosis, or treatment**. Always consult a qualified healthcare provider with any questions about a medical condition.
