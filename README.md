
# ValyGate

**Production‑grade LLM Gateway, Proxy, and Observability Platform**  
*Rust‑powered, high‑throughput proxy + Langfuse‑style observability in one.*

![CodeRabbit Pull Request Reviews](https://img.shields.io/coderabbit/prs/github/CLoaKY233/Valygate?utm_source=oss&utm_medium=github&utm_campaign=CLoaKY233%2FValygate&labelColor=171717&color=FF570A&link=https%3A%2F%2Fcoderabbit.ai&label=CodeRabbit+Reviews)

## What it does

| Feature | Status | Description |
|---------|--------|-------------|
| **HTTP Proxy** | 🟢 Working | OpenAI‑compatible `/v1/chat/completions` → provider forwarding |
| **Model Routing** | 🟡 Planned | `gpt‑4` → OpenAI, `sonnet` → Anthropic |
| **Observability** | 🟢 Working | Request‑level tracing, Prometheus metrics (`latency_p95`, `errors_total`) |
| **Rate Limiting** | 🟡 Planned | Per‑API‑key, per‑tenant quotas |
| **Prompt Mgmt** | 🔴 Planned | Versioning, A/B testing, deployment |



