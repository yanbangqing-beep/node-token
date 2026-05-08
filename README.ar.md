<div align="center">

# node-token

<p align="center">
  <a href="./README.md">English</a> |
  <a href="./README.zh-CN.md">简体中文</a> |
  <a href="./README.zh-TW.md">繁體中文</a> |
  <a href="./README.es.md">Español</a> |
  <a href="./README.ar.md">العربية</a>
</p>

**عميل عقدة الحاسوب الشخصي KeyCompute — اجلب قوتك الحاسوبية**

<p align="center">
  <a href="https://github.com/keycompute/node-token/stargazers"><img src="https://img.shields.io/github/stars/keycompute/node-token?style=social" alt="GitHub Stars" /></a>
  <a href="https://github.com/keycompute/node-token/issues"><img src="https://img.shields.io/github/issues/keycompute/node-token" alt="GitHub Issues" /></a>
  <a href="./LICENSE"><img src="https://img.shields.io/badge/License-GPLv3-blue.svg" alt="GPLv3 License" /></a>
  <a href="./CONTRIBUTING.md"><img src="https://img.shields.io/badge/PRs-welcome-brightgreen" alt="PRs Welcome" /></a>
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/Rust-1.92%2B-orange?logo=rust" alt="Rust Version" /></a>
</p>

<p align="center">
  <a href="#الميزات">الميزات</a> •
  <a href="#البدء-السريع">البدء السريع</a> •
  <a href="#الإعدادات">الإعدادات</a> •
  <a href="#الاستخدام">الاستخدام</a>
</p>

</div>

---

## نظرة عامة

`node-token` هو عميل Rust خفيف الوزن يعمل على أجهزة الكمبيوتر الشخصية ويربطها بمنصة [KeyCompute](https://github.com/keycompute/keycompute) كعقد حوسبة. يقوم باستطلاع الخادم للحصول على المهام، وتنفيذها على نسخة Ollama المحلية، وإرسال النتائج — كل ذلك دون الحاجة إلى عنوان IP عام.

---

## الميزات

- **استطلاع قائم على السحب**: يعمل خلف NAT والشبكات المنزلية دون حاجة إلى IP عام
- **تنفيذ محلي عبر Ollama**: تشغيل النماذج المستضافة على Ollama مباشرة على جهازك
- **استرداد تلقائي**: يحفظ حالة الجلسة محليًا ويستأنف بعد إعادة التشغيل
- **نبضات قلب دورية**: تحافظ نبضات القلب الدورية على توفر العقدة
- **إيقاف تدريجي**: يتوقف عن قبول المهام الجديدة عند الخروج مع إكمال العمل الجاري
- **معالجة حالة الاستبعاد**: يعكس حالة الاستبعاد من الخادم ويواصل نبضات القلب منخفضة التردد لرؤية المشرف

---

## المتطلبات الأساسية

| المكوّن | الإصدار |
|:---|:---|
| Rust | ≥ 1.92 |
| Ollama | الأحدث |

> تحتاج إلى نسخة Ollama قيد التشغيل مع نموذج واحد على الأقل تم تنزيله. يقوم العميل بفحص النماذج المحلية عند بدء التشغيل ويبلغ عنها أثناء التسجيل.

---

## البدء السريع

### تثبيت Ollama

```bash
# Linux
curl -fsSL https://ollama.com/install.sh | sh

# تنزيل نموذج
ollama pull gemma3:270m
```

### بناء وتشغيل node-token

```bash
# استنساخ وبناء
git clone https://github.com/keycompute/node-token.git
cd node-token
cp config.example.toml config.toml
# عدّل config.toml بعنوان خادم KeyCompute ورمز التسجيل

# بناء
cargo build --release

# تشغيل
./target/release/node-token
```

### Docker

باستخدام `docker-compose.yml` (موصى به، يتضمن Ollama وتسخين النموذج):

```bash
# إنشاء .env من القالب (عدّل NODE_TOKEN__REGISTRATION_TOKEN)
cp .env.example .env

# بدء Ollama + node-token
docker compose up -d

# متابعة السجلات
docker compose logs -f
```

تشغيل node-token منفردًا (يتطلب وجود نسخة Ollama قيد التشغيل):

```bash
# بناء الصورة
docker build -t node-token .

# إنشاء وحدة تخزين بيانات
docker volume create node_token_data

# تشغيل (استخدم --network host للوصول إلى Ollama على المضيف)
docker run -d \
  --name node-token \
  --network host \
  -v node_token_data:/data \
  -e NODE_TOKEN__SERVER_URL="http://keycompute-server:3000" \
  -e NODE_TOKEN__REGISTRATION_TOKEN="رمز-التسجيل-الخاص-بك" \
  -e NODE_TOKEN__CLIENT_INSTANCE_ID="عقدتي-001" \
  -e NODE_TOKEN__DISPLAY_NAME="عقدة حاسوبي" \
  -e NODE_TOKEN__OLLAMA_URL="http://localhost:11434" \
  node-token
```

---

## الإعدادات

يتم تحميل الإعدادات من `config.toml` (أو مسار يتم تعيينه عبر متغير البيئة `NODE_TOKEN_CONFIG`). متغيرات البيئة ذات البادئة `NODE_TOKEN__` تتجاوز قيم الملف.

| المتغير | الوصف | الافتراضي | مطلوب |
|:---|:---|:---|:---:|
| `server_url` | عنوان خادم KeyCompute | `http://localhost:3000` | ✅ |
| `registration_token` | رمز التسجيل من مسؤول KeyCompute | — | ✅ |
| `client_instance_id` | معرّف فريد لهذه العقدة (يستمر عبر إعادة التشغيل) | — | ✅ |
| `display_name` | اسم مقروء للعقدة | — | ✅ |
| `ollama_url` | نقطة نهاية API المحلية لـ Ollama | `http://localhost:11434` | ⚪ |
| `heartbeat_interval_secs` | فاصل نبضات القلب بالثواني | `30` | ⚪ |
| `excluded_poll_check_interval_secs` | فاصل فحص الاستطلاع عند الاستبعاد | `30` | ⚪ |
| `data_dir` | دليل البيانات المحلي لاستمرارية الجلسة | `~/.local/share/node-token` | ⚪ |

**تعيين متغيرات البيئة**: `NODE_TOKEN__SERVER_URL`، `NODE_TOKEN__REGISTRATION_TOKEN`، إلخ.

> لا يتم تسجيل `registration_token` و `session_token` أبدًا كنص صريح.

---

## الاستخدام

بمجرد تسجيل `node-token` وتشغيله، يمكن للمستخدمين إرسال الطلبات عبر واجهة برمجة تطبيقات KeyCompute باستخدام بادئة النموذج `node:`:

```bash
curl -s http://خادم-keycompute:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer sk-xxx" \
  -d '{
    "model": "node:gemma3:270m",
    "messages": [{"role": "user", "content": "مرحبًا!"}],
    "stream": false
  }'
```

- `node:<اسم_النموذج>` يوجه الطلب إلى مجموعة العقد (بدون بث فقط)
- `<اسم_النموذج>` (بدون بادئة) يوجه إلى مسار حسابات المزوّد العادي

---

## آلية العمل

```text
┌─────────────┐    استطلاع المهام     ┌──────────────────┐
│  node-token │ ◄────────────────── │  KeyCompute       │
│  (جهازك)    │ ──────────────────► │  الخادم           │
│             │  نبضات/إكمال         │                   │
│     │       │                      │        │          │
│     │ استدعاء│                    │        │ إدراج    │
│     ▼       │                      │        ▼          │
│  ┌───────┐  │                      │  ┌──────────┐    │
│  │Ollama │  │                      │  │ API      │    │
│  │       │  │                      │  │ المستخدم │    │
│  └───────┘  │                      │  └──────────┘    │
└─────────────┘                      └──────────────────┘
```

1. يسجل `node-token` مع خادم KeyCompute، ويبلغ عن نماذج Ollama المتاحة
2. يرسل نبضات قلب دورية للحفاظ على نشاط الجلسة
3. يقوم باستطلاع طويل للمهام المطابقة لنماذجه المقبولة
4. عند استلام مهمة، يستدعي نسخة Ollama المحلية ويرسل النتيجة
5. إذا تم استبعاده من قبل الخادم (مثلاً بسبب كثرة الفشل)، يتوقف عن الاستطلاع لكنه يواصل نبضات القلب منخفضة التردد

---

## التطوير

```bash
# بناء
cargo build --release

# تشغيل الاختبارات
cargo test --lib
cargo test --tests

# فحوصات الكود
cargo clippy --all-targets -- -D warnings
cargo fmt --all --check
```

---

## هيكل المشروع

```text
node-token/
├── src/
│   ├── main.rs              # نقطة الدخول، معالجة الإشارات
│   ├── config.rs            # إدارة الإعدادات
│   ├── error.rs             # أنواع الأخطاء
│   ├── lib.rs               # جذر المكتبة
│   ├── client/              # عملاء HTTP
│   │   ├── api.rs           # عميل KeyCompute API
│   │   └── ollama.rs        # عميل Ollama HTTP
│   ├── protocol/            # أنواع البروتوكول (منسوخة من keycompute-types)
│   │   ├── types.rs         # DTOs بروتوكول العقدة
│   │   └── ollama.rs        # أنواع Ollama API
│   ├── runtime/             # منطق وقت التشغيل الأساسي
│   │   ├── register.rs      # منطق التسجيل
│   │   ├── heartbeat.rs     # حلقة نبضات القلب
│   │   ├── poll.rs          # حلقة الاستطلاع
│   │   └── executor.rs      # منفذ المهام
│   └── storage/             # الاستمرارية المحلية
│       └── mod.rs           # تخزين الجلسة
├── tests/                   # اختبارات التكامل
├── benches/                 # اختبارات الأداء
├── config.example.toml
├── .env.example
└── Cargo.toml
```

---

## الترخيص

هذا المشروع متاح بموجب ترخيص [GNU GPLv3](LICENSE).

---

<div align="center">

### 💖 شكرًا لاستخدام node-token

إذا كان هذا المشروع مفيدًا لك، فلا تتردد في منحه ⭐️ نجمة.

**[البدء السريع](#البدء-السريع)** • **[الإبلاغ عن المشكلات](https://github.com/keycompute/node-token/issues)** • **[أحدث الإصدارات](https://github.com/keycompute/node-token/releases)**

</div>
