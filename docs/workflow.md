# Workflow & Technical Boundaries

Dokumen ini berfungsi sebagai acuan teknis (Technical Compass) untuk pengembangan fitur. Segala implementasi kode **harus** mengacu pada batasan flow di bawah ini untuk menjaga privasi user dan integritas produk.

## A. User Flow

1.  **Onboarding**
    * User masuk tanpa login paksa (Guest mode first).
    * Penjelasan singkat: "Jurnal refleksi aman, AI opsional."
    * *Action:* User setuju Terms & Privacy.

2.  **Daily Check-in (Core - Free)**
    * Input: Text (jurnal) atau Mood selector.
    * Storage: Disimpan secara **lokal** di device user.
    * Tidak ada trigger ke server AI.

3.  **Data Handling**
    * Default: Data stay di device (Local-first).
    * Sync (Optional): Hanya jika user login untuk backup (End-to-End Encrypted jika memungkinkan).

4.  **AI Reflection (Premium Feature)**
    * User klik tombol "Reflect with AI" secara sadar (Active Trigger).
    * User melihat prompt konfirmasi: "Data jurnal ini akan dikirim sementara untuk diproses AI."

## B. Data Flow Architecture

### Free User
* **Flow:** Input → Local DB (SQLite/Realm/etc).
* **Server Access:** Zero. Server hanya terima data anonim metrik (jika ada consent) untuk crash reporting.

### Premium User (AI Feature)
* **Flow:** Input → User Consent → API Gateway → LLM Processing → Result → Client.
* **Data Retention:**
    * Data dikirim *hanya* saat request.
    * **NO Long-term Memory:** AI tidak boleh menyimpan konteks antar sesi (Stateless).
    * Setelah response diterima, data di server AI/Backend harus di-flush/tidak dilog.

## C. AI Workflow Rules

1.  **OFF by Default:** Fitur AI tidak boleh jalan otomatis di background.
2.  **Role:** AI bertindak sebagai "Cermin" (Reflection), bukan "Guru" atau "Dokter" (Advice).
3.  **Output Guardrails:**
    * AI dilarang memberikan diagnosis medis.
    * AI dilarang memberikan instruksi preskriptif ("Kamu harus melakukan X").
    * AI fokus bertanya balik atau merangkum perasaan.
