---
name: Deployment Help
about: Get help with installing or deploying Campfire
title: '[DEPLOYMENT] '
labels: ['deployment', 'help-wanted']
assignees: ''
---

## 🚀 Deployment Issue
**What deployment method are you having trouble with?**
- [ ] Local installation (curl script)
- [ ] Railway deployment
- [ ] Docker deployment
- [ ] Building from source
- [ ] Custom deployment

## 📋 What You Tried
**Describe what you attempted to do:**

### Installation Command
```bash
# Paste the exact command you ran
```

### Expected Result
**What did you expect to happen?**

### Actual Result
**What actually happened?**

## ❌ Error Messages
**Paste any error messages you encountered:**

### Installation Script Errors
```
Paste installation script output here
```

### Railway Deployment Errors
```
Paste Railway build/deployment logs here
```

### Browser Errors
```
Paste browser console errors here (F12 → Console)
```

## 🖥️ Environment Details

### Local Environment
- **Operating System**: [e.g. macOS 13.1, Ubuntu 22.04, Windows 11]
- **Architecture**: [e.g. x86_64, ARM64/Apple Silicon]
- **Shell**: [e.g. bash, zsh, PowerShell]
- **Available Tools**: [e.g. curl, wget, git, docker]

### Network Environment
- **Corporate Network**: Yes/No
- **VPN**: Yes/No
- **Firewall Restrictions**: Yes/No/Unknown
- **Proxy Settings**: Yes/No

### Railway Deployment (if applicable)
- **Railway Region**: [e.g. us-west1, eu-west1]
- **Deployment Method**: [e.g. GitHub integration, CLI]
- **Custom Domain**: Yes/No

## 🔍 Troubleshooting Attempted
**What troubleshooting steps have you already tried?**
- [ ] Checked internet connection
- [ ] Tried different browser
- [ ] Cleared browser cache
- [ ] Restarted terminal/computer
- [ ] Tried manual download
- [ ] Checked firewall settings
- [ ] Read troubleshooting section in README
- [ ] Searched existing issues

## 🎯 Goal
**What are you ultimately trying to achieve?**
- [ ] Try Campfire locally for evaluation
- [ ] Deploy for small team (< 10 people)
- [ ] Deploy for larger team (10+ people)
- [ ] Set up development environment
- [ ] Production deployment with custom domain

## ⏰ Timeline
**How urgent is this for you?**
- [ ] Urgent - Need working today
- [ ] Soon - Need within a few days
- [ ] Flexible - When convenient

## 🤝 Assistance Level
**What level of help do you need?**
- [ ] Just point me in the right direction
- [ ] Step-by-step guidance
- [ ] Screen sharing / video call assistance
- [ ] Someone to deploy it for me

## 📞 Contact Preference
**How would you prefer to receive help?**
- [ ] GitHub comments (public)
- [ ] Email (private)
- [ ] Discord/Slack chat
- [ ] Video call

---

**We're here to help! 🔥**

*Deployment issues are usually resolved quickly. For urgent problems, email campfire-support@that-in-rust.dev*

**Common Quick Fixes:**
- **Port 3000 busy**: Change `CAMPFIRE_PORT=3001` in `~/.campfire/.env`
- **Permission denied**: Run `chmod +x ~/.local/bin/campfire-on-rust`
- **Command not found**: Add `export PATH="$PATH:$HOME/.local/bin"` to your shell config
- **Railway timeout**: Try redeploying or contact Railway support