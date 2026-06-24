# Жишээ файлууд (https://filename24.github.io/shalgalt-omr/mn/docs/user/samples)



Програмыг туршихад зориулсан **гурван бэлэн жишээ төсөл**.

<Callout type="info" title="Анхаар">
  Эдгээр нь зөвхөн жишээ бөгөөд **жинхэнэ сурагчийн мэдээлэл агуулаагүй**.
</Callout>

| Файл                        | Тайлбар                                                                     | Нууц үг         |
| --------------------------- | --------------------------------------------------------------------------- | --------------- |
| `sample-open.shalgalt`      | Шифрлээгүй, PDF-гүй (хамгийн жижиг). Ямар ч zip програмаар нээж үзэж болно. | —               |
| `sample-with-pdf.shalgalt`  | Шифрлээгүй, хэвлэх `exam.pdf`-тэй.                                          | —               |
| `sample-encrypted.shalgalt` | Нууц үгээр шифрлэсэн, PDF-тэй.                                              | `shalgalt-2026` |

## Татаж авах [#татаж-авах]

Файлуудыг репозиторын `docs/user/samples/` хавтаснаас үзэж/татаж авна:

<Cards>
  <Card href="https://github.com/filename24/shalgalt-omr/tree/stable/docs/user/samples" title="GitHub дээр үзэх">
    Жишээ файлуудыг репозитор дотор нээж үзэх.
  </Card>

  <Card href="https://raw.githubusercontent.com/filename24/shalgalt-omr/stable/docs/user/samples/sample-open.shalgalt" title="sample-open.shalgalt">
    Шифрлээгүй, PDF-гүй жишээг шууд татах.
  </Card>

  <Card href="https://raw.githubusercontent.com/filename24/shalgalt-omr/stable/docs/user/samples/sample-with-pdf.shalgalt" title="sample-with-pdf.shalgalt">
    Хэвлэх PDF-тэй жишээг шууд татах.
  </Card>

  <Card href="https://raw.githubusercontent.com/filename24/shalgalt-omr/stable/docs/user/samples/sample-encrypted.shalgalt" title="sample-encrypted.shalgalt">
    Нууц үгээр шифрлэсэн жишээг шууд татах.
  </Card>
</Cards>

## Бүтэц [#бүтэц]

Жишээ төсөл бүр дараах хэсгүүдийг агуулна:

<Files>
  <File name="manifest.json" />

  <File name="template.json" />

  <File name="answer-keys.json" />

  <File name="metadata.json" />

  <File name="students.csv" />

  <File name="exam.pdf" />
</Files>

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
