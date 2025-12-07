# GitHub Pages Quick Reference Card

**One-page cheat sheet for common tasks**

---

## 🔗 Your Live URLs

```
Blog:    https://jefh507.github.io/org-chart-goose-orchestrator/blog/enterprise-ai-orchestration-privacy-first.html
Home:    https://jefh507.github.io/org-chart-goose-orchestrator/
Repo:    https://github.com/JEFH507/org-chart-goose-orchestrator
```

---

## ✏️ Update Existing Blog Post

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin
nano docs/blog/enterprise-ai-orchestration-privacy-first.md
git add docs/blog/ && git commit -m "docs: Update blog post" && git push origin main
# Wait 1-2 min, then force-refresh browser (Ctrl+Shift+R)
```

---

## 📝 Create New Blog Post

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin
nano docs/blog/my-new-post.md

# Write your content in Markdown, then:
git add docs/blog/my-new-post.md
git commit -m "docs: Add new blog post"
git push origin main

# Visit: https://jefh507.github.io/org-chart-goose-orchestrator/blog/my-new-post.html
```

---

## 📸 Add Images

```bash
# 1. Copy image to images folder
cp ~/Pictures/screenshot.png docs/blog/images/

# 2. Reference in Markdown (relative path!)
# ![Caption](images/screenshot.png)

# 3. Commit and push
git add docs/blog/
git commit -m "docs: Add screenshot"
git push origin main
```

**⚠️ Image Path Rules:**
- ✅ `images/screenshot.png` (relative, correct!)
- ❌ `/images/screenshot.png` (absolute, wrong!)
- ❌ `../images/screenshot.png` (wrong!)

---

## 🔍 Verify Changes

```bash
# Check GitHub Pages build status
open https://github.com/JEFH507/org-chart-goose-orchestrator/actions

# Test image loading
curl -I https://jefh507.github.io/org-chart-goose-orchestrator/blog/images/YOURIMAGE.png
# Should return: HTTP/2 200 (success!)
```

---

## 📖 Full Documentation

**Location:** `docs/guides/github-pages/`

| File | Purpose |
|------|---------|
| `README.md` | Overview + quick start |
| `GITHUB_PAGES_SETUP.md` | How Jekyll works, file structure |
| `GITHUB_PAGES_LIVE.md` | URLs for sharing |
| `PUBLICATION_CHECKLIST.md` | Quality checks |
| `IMAGES_FIXED.md` | Image fix history |

---

## 🚨 Common Issues

**Images not loading?**
- Check: Relative path `images/file.png` (NOT `/images/`)
- Check: File exists in `docs/blog/images/`
- Force-refresh: Ctrl+Shift+R

**Changes not showing?**
- Wait: 1-2 minutes for rebuild
- Check: GitHub Actions (link above)
- Force-refresh: Clear cache

**Build failed?**
- Check: `_config.yml` valid YAML
- Check: No syntax errors in Markdown
- Check: Branch is `main`, folder is `/docs`

---

## 💡 Pro Tips

**Markdown file naming:**
- `my-post.md` → `/blog/my-post.html`
- `2025-12-06-title.md` → `/blog/2025-12-06-title.html`
- Use lowercase, hyphens (not spaces)

**Image organization:**
- All images in `docs/blog/images/`
- Shared across all blog posts
- Name descriptively: `architecture-diagram.png`

**Git workflow:**
1. Edit files locally
2. `git add`, `git commit`, `git push`
3. GitHub Pages auto-rebuilds
4. Force-refresh browser to see changes

---

**Last Updated:** December 6, 2025  
**Full Docs:** `docs/guides/github-pages/README.md`
