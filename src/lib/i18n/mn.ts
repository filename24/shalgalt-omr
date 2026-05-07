/**
 * Mongolian (Cyrillic) UI string table.
 *
 * Single source of truth for every user-visible label, message, and toast in the app.
 * Per CLAUDE.md "Language Conventions": code identifiers, comments, and `AppError.code`
 * stay English; everything the end user sees lives here.
 *
 * Use either form at the call site:
 *   - direct nested access: `mn.editor.toolbar.save`
 *   - dotted path lookup via `t(...)` (see `./index.ts`) for dynamic keys
 *     such as `t(`status.stage.${stage}`)`.
 */
export const mn = {
  app: {
    title: "Шалгалт OMR",
  },
  nav: {
    group: "Ажлын талбар",
    dashboard: "Хяналтын самбар",
    editor: "Загвар засварлагч",
    grade: "PDF шалгах",
    results: "Үр дүн",
  },
  status: {
    idle: "Сул зогсолт",
    working: "Ажиллаж байна",
    job: "ажил",
    template: "загвар",
    stage: {
      loading_pdf: "PDF ачааллаж байна",
      rasterizing: "Растер хөрвүүлэлт",
      detecting_markers: "Маркер илрүүлэлт",
      reading_bubbles: "Бөмбөлөг уншилт",
      grading: "Шалгаж байна",
      saving: "Хадгалж байна",
      done: "Дууссан",
      failed: "Амжилтгүй",
    },
  },
  editor: {
    title: "Загвар засварлагч",
    untitled: "Шинэ загвар",
    presets: {
      standard: "Стандарт сорил",
      empty: "Хоосон",
      standardDefaultTitle: "Стандарт сорил",
      sections: {
        shifr: "Шифр",
        variant: "Хувилбар",
        section1: "1-Р ХЭСЭГ",
        section21: "2.1",
        section22: "2.2",
        other: "Бусад",
      },
      labels: {
        shifrRow: "Шифр",
        variant: "Хувилбар",
        question: "Q",
      },
    },
    empty: {
      noBackdrop: "Зургийн дэвсгэр сонгоно уу",
      noBackdropHint: "Сканнердсан хоосон бланкны зураг эсвэл PDF-ийн эхний хуудсыг ашиглана.",
      noBackdropCta: "Зургийн дэвсгэр сонгох",
    },
    toolbar: {
      new: "Шинэ",
      open: "Нээх",
      save: "Хадгалах",
      saveAs: "Өөр нэрээр хадгалах",
      importBackdrop: "Зургийн дэвсгэр",
      importBackdropImage: "Зураг сонгох",
      importBackdropPdf: "PDF-ийн эхний хуудас",
      addGroup: "Бөмбөлгийн бүлэг нэмэх",
      deleteSelected: "Сонгосныг устгах",
      undo: "Буцаах",
      redo: "Дахин хийх",
      saveDisabledNoBackdrop: "Эхлээд зургийн дэвсгэр сонгоно уу",
      undoRedoDeferredHint: "P5 шатанд нэмэгдэнэ",
    },
    markers: {
      tl: "Зүүн дээд",
      tr: "Баруун дээд",
      br: "Баруун доод",
      bl: "Зүүн доод",
    },
    groups: {
      kindStudentId: "Сурагчийн дугаар",
      kindQuestion: "Асуулт",
      defaultLabel: "Шинэ бүлэг",
    },
    layers: {
      markers: "Маркерууд",
      groups: "Бөмбөлгийн бүлгүүд",
      empty: "Бүлэг алга байна",
    },
    inspector: {
      templateTitle: "Загварын нэр",
      groupLabel: "Бүлгийн нэр",
      groupKind: "Төрөл",
      bubbleCount: "Бөмбөлгийн тоо",
      direction: "Чиглэл",
      directionHorizontal: "Хэвтээ",
      directionVertical: "Босоо",
      spacing: "Зай",
      answerIndex: "Зөв хариулт",
      answerIndexHint: "0-оос эхлэн дугаарлагдсан",
      score: "Оноо",
      manualLayoutBadge: "Гар тохиргоо",
      noSelection: "Зүүн талаас бүлэг сонгоно уу",
    },
    toasts: {
      saved: "Загвар хадгалагдлаа",
      saveFailed: "Хадгалах үед алдаа гарлаа",
      backdropImported: "Зургийн дэвсгэр оруулагдлаа",
      backdropImportFailed: "Зургийн дэвсгэр оруулахад алдаа гарлаа",
    },
  },
  dialog: {
    unsavedChanges: {
      title: "Хадгалаагүй өөрчлөлт байна",
      body: "Үргэлжлүүлбэл одоогийн өөрчлөлтүүд устах болно. Үргэлжлүүлэх үү?",
      confirm: "Үргэлжлүүлэх",
      cancel: "Болих",
    },
    saveAs: {
      title: "Өөр нэрээр хадгалах",
      body: "Шинэ загварын нэрийг оруулна уу.",
      confirm: "Хадгалах",
      cancel: "Болих",
    },
    confirmDelete: {
      title: "Устгах уу?",
      body: "Сонгосон бүлэг устах болно.",
      confirm: "Устгах",
      cancel: "Болих",
    },
  },
  errors: {
    unknown: "Үл мэдэгдэх алдаа гарлаа",
    pdfium_unavailable: "PDF боловсруулах сан олдсонгүй",
    template_corrupt: "Загварын файл эвдэрсэн байна",
    backdrop_required: "Зургийн дэвсгэр заавал шаардлагатай",
  },
} as const;
