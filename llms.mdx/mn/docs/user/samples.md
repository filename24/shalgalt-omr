# Жишээ файлууд (https://filename24.github.io/shalgalt-omr/mn/docs/user/samples)



# Жишээ `.shalgalt` файлууд [#жишээ-shalgalt-файлууд]

Програмыг туршихад зориулсан **гурван бэлэн жишээ төсөл**. Эдгээр нь зөвхөн жишээ бөгөөд
**жинхэнэ сурагчийн мэдээлэл агуулаагүй**.

| Файл                        | Тайлбар                                                                     | Нууц үг         |
| --------------------------- | --------------------------------------------------------------------------- | --------------- |
| `sample-open.shalgalt`      | Шифрлээгүй, PDF-гүй (хамгийн жижиг). Ямар ч zip програмаар нээж үзэж болно. | —               |
| `sample-with-pdf.shalgalt`  | Шифрлээгүй, хэвлэх `exam.pdf`-тэй.                                          | —               |
| `sample-encrypted.shalgalt` | Нууц үгээр шифрлэсэн, PDF-тэй.                                              | `shalgalt-2026` |

## Татаж авах [#татаж-авах]

Файлуудыг репозиторын `docs/user/samples/` хавтаснаас үзэж/татаж авна:

* [Жишээ файлуудыг GitHub дээр үзэх](https://github.com/filename24/shalgalt-omr/tree/stable/docs/user/samples)
* Шууд татах:
  [sample-open.shalgalt](https://raw.githubusercontent.com/filename24/shalgalt-omr/stable/docs/user/samples/sample-open.shalgalt)
  ·
  [sample-with-pdf.shalgalt](https://raw.githubusercontent.com/filename24/shalgalt-omr/stable/docs/user/samples/sample-with-pdf.shalgalt)
  ·
  [sample-encrypted.shalgalt](https://raw.githubusercontent.com/filename24/shalgalt-omr/stable/docs/user/samples/sample-encrypted.shalgalt)

## Бүтэц [#бүтэц]

Жишээ төсөл бүр дараах хэсгүүдийг агуулна:

* `manifest.json` — гарчиг, огноо, хуудасны тоо (үргэлж задгай).
* `template.json` — 5 асуулттай (A–D) загвар.
* `answer-keys.json` — A, B хоёр хувилбарын зөв хариулт.
* `metadata.json`, `students.csv` — хичээл, анги, сурагчдын жагсаалт.
* `exam.pdf` — (зөвхөн PDF-тэй хувилбарт) хэвлэх хуудас.

## Дахин үүсгэх [#дахин-үүсгэх]

Эдгээр файлыг кодоос дахин үүсгэхдээ:

```bash
cargo run -p shalgalt-fileformat --example generate_samples
```

Эх өгөгдөл нь `docs/user/samples/_payload/` дотор задгай байдлаар хадгалагдсан тул засаад
дахин үүсгэж болно.
