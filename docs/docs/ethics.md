# Ethics & Project Boundaries

Dokumen ini mendefinisikan posisi etis proyek. Semua kontributor dan fitur baru wajib mematuhi batasan ini.

## 1. Identity: IS / IS NOT

| This Project **IS** | This Project **IS NOT** |
| :--- | :--- |
| ✅ **Wellbeing Support:** Alat bantu refleksi diri. | ❌ **Medical Device:** Bukan pengganti diagnosa klinis. |
| ✅ **Reflective Tool:** Cermin untuk pikiran user. | ❌ **Diagnostic Tool:** Tidak melabeli gangguan mental. |
| ✅ **Privacy-First:** User kontrol penuh data mereka. | ❌ **Therapy/Treatment:** Bukan sesi terapi pengganti profesional. |
| ✅ **User-Led:** User yang memulai interaksi. | ❌ **Passive Surveillance:** Tidak memantau user diam-diam. |

## 2. Core Principles

### A. Consent & Control
* User harus selalu tahu kapan AI bekerja.
* Tidak ada data yang dikirim ke cloud tanpa aksi klik eksplisit dari user.
* User berhak menghapus seluruh data mereka (lokal & cloud) kapan saja ("Right to be forgotten").

### B. Data Ownership
* Data jurnal adalah milik user, bukan milik platform.
* Kita tidak menjual data user ke pihak ketiga.
* Kita tidak menggunakan data user untuk melatih (training) model AI publik tanpa izin eksplisit terpisah.

## 3. AI Limitations & Safety
* **No Hallucinations on Health:** AI harus diprogram untuk menolak menjawab jika user bertanya tentang dosis obat atau diagnosa penyakit.
* **Crisis Intervention:** Jika terdeteksi kata kunci bahaya (self-harm), sistem harus mematikan respon AI dan menampilkan nomor darurat/bantuan profesional (Hard-coded rule).
