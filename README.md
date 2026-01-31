# Workflow – AI Mental Health Chatbot

This document defines the product workflow, team responsibilities, and feature boundaries for the AI Mental Health Chatbot.

---

## 1. Purpose
- Serve as a technical and product compass for development
- Keep frontend, backend, and infrastructure aligned
- Prevent feature creep and ethical violations

---

## 2. Team Responsibilities

### Product & Ethics Owner
**(Zavy)**
- Write and maintain:
  - `workflow.md`
  - `ethics.md`
- Define and approve feature scope
- Review user flow and ethical risks
- Final approval before production release

---

### Engineering
**(Gilang)**
- Repository setup and project structure
- Frontend development (React + Vite)
- Backend development (Rust + Axum)
- API, database, and LLM integration
- Deployment and server maintenance

---

## 3. High-Level Architecture
User (WebUI)
↓
Frontend (React + Vite)
↓ REST API
Backend SDK (Rust + Axum)
↓
OpenRouter (LLM)
MongoDB (Data)

- Frontend acts as a **UI layer only**
- All AI logic and data processing reside in the backend
- No LLM keys are exposed on the frontend

---

## 4. Core User Flow

### 4.1 Onboarding
- User accesses the WebUI
- Clear notice that:
  - The chatbot is **not a medical professional**
  - It does **not replace professional mental health services**
- User agrees before starting the chat

---

### 4.2 Chat Interaction
1. User sends a message
2. Frontend forwards the request to the backend
3. Backend:
   - Validates input
   - Retrieves context if available
   - Sends prompt to OpenRouter
4. AI response is returned to the frontend
5. Frontend renders the response (Markdown supported)

---

### 4.3 Data Handling
- Minimal data collection (messages and responses)
- No explicit personal identification stored
- User data is **not used to retrain models**

---

## 5. Feature Scope

### In Scope
- Text-based one-on-one chat
- Lightweight mental health support
- Markdown rendering
- Technical logging (errors and performance)

---

### Out of Scope
- Medical or psychological diagnosis
- Clinical assessment or labeling
- Emergency or crisis intervention
- Voice or video interaction

---

## 6. Deployment Flow
- Backend and frontend are deployed by Engineering
- Configuration handled via environment variables
- Production updates require:
  - Review by Product & Ethics Owner
  - Compliance with `ethics.md`

---

## 7. Development Principles
- Simplicity over complexity
- Safety over features
- Clarity over speed

