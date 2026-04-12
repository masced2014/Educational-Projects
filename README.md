# Educational-Projects

This repository collects a series of learning projects. The goal is to explore new technologies, methods, and tools through hands-on practice and to document the results clearly.

## 📚 Project Overview

Each subfolder contains an individual learning project with an own description of the learning goal.

### Current Projects

- **agentic_ai**: Agentic AI chatbot built with LangGraph and Ollama, running fully locally.
  - `multi-agent-chatbot.ipynb` — routes topics to specialist agents (HealthBot, QualityBot) via an LLM classifier
  - `app.py` — standalone Gradio browser UI for the same multi-agent system
  - `add_to_corpus.py` — utility to index additional Wikipedia articles into the local ChromaDB RAG corpus
  - Project documentation: [agentic_ai/README.md](agentic_ai/README.md)
- **data_science**: Exploratory data analysis of 1,399 Roman mining sites from the OXREP database using machine learning and statistical methods (EDA, data cleaning, visualization, classification).
  - `Roman_Mining_EDA_Analysis.ipynb` — CRISP-DM based analysis covering regional distribution, mining techniques, site complexity, and a Random Forest classifier
  - Project documentation: [data_science/README.md](data_science/README.md)
- **rust_file_encrypt**: Rust CLI project for file encryption/decryption using AES-256-GCM and Argon2id. **Built entirely with GitHub Copilot** as an AI-assisted development exercise.
  - Project documentation: [rust_file_encrypt/README.md](rust_file_encrypt/README.md)
  - Coverage notes: [rust_file_encrypt/COVERAGE.md](rust_file_encrypt/COVERAGE.md)
  - Supply chain security: SBOM generated with Syft and scanned with Grype on every PR that touches `rust_file_encrypt` — see the [SBOM section](rust_file_encrypt/README.md#supply-chain-security-sbom)

## 🧠 Learning Philosophy

- Small, self-contained projects
- Clean, readable, and reproducible code
- Documentation as an essential part of the learning process
- Active use of AI tools (GitHub Copilot) to explore AI-assisted development

## 🛠️ Technologies

Depending on the project, different tools and libraries are used.

## 🚀 How to Use These Projects

- Explore the code and notebooks
- Run the examples locally
- Adapt ideas for your own learning
- Share feedback or start discussions

## 📄 License

This repository is intended for educational purposes.
License: MIT
