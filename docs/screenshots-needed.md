# Screenshots needed for the user manual

This is a maintainer checklist (English, intentionally outside the docs content tree so it
does not appear in the site navigation). Each user-manual page marks screenshot slots with a
`📷` placeholder blockquote. Capture the images below from the running app, place them under
`docs/site/public/img/`, and replace the placeholder blockquotes with standard image embeds.

Capture both relevant breakpoints where layout differs, and prefer the **Mongolian** UI
(the app ships Mongolian-only).

| File name (`docs/site/public/img/`) | Page | Route in app | What to show |
| ----------------------------------- | ---- | ------------ | ------------ |
| `dashboard.png` | quick-start, editor | `/` | Dashboard with quick-action widgets and recent lists |
| `editor-toolbar.png` | quick-start, editor | `/editor` | Editor toolbar (new/open/save/undo/redo/export) + a bubble group |
| `editor-bubble-group.png` | editor | `/editor` | A placed bubble group with markers visible |
| `exam-detail-answer-key.png` | quick-start, exams | `/exams/[id]` | Exam detail with the answer-key editor open |
| `grade-progress.png` | quick-start, grading | `/grade` | File selection + variant picker + progress bar mid-run |
| `review-canvas.png` | review | `/review/[job_id]` | Manual review: thumbnail list + bubble overlay on a sheet |
| `results-detail.png` | export | `/results/[job_id]` | Result detail: student list + per-question breakdown |
| `project-open-passphrase.png` | project-files | `/exams/import` | Open-project flow with the passphrase prompt |

## How to wire an image once captured

Replace, e.g.:

```md
> 📷 *Дэлгэцийн зураг: ...*
```

with:

```md
![Alt text in the page's language](/img/editor-toolbar.png)
```

(The leading `/img/...` resolves under the configured `basePath` via the docs site's asset
handling.)
