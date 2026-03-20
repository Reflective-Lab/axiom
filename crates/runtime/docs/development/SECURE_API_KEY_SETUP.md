# Secure API Key Setup for Testing

This guide explains how to safely provide API keys for running integration tests in Cursor.

---

## ⚠️ Security Principles

1. **Never commit API keys to git**
2. **Never hardcode keys in source code**
3. **Use environment variables or secure storage**
4. **Rotate keys if accidentally exposed**

---

## Method 1: Environment Variables (Recommended)

### Option A: Set in Terminal (Session Only)

```bash
# Set for current terminal session
export ANTHROPIC_API_KEY="sk-ant-api03-..."

# Run tests
cargo test --test integration_prompt_formatting -- --ignored
```

**Pros:**
- ✅ Simple and immediate
- ✅ No files to manage
- ✅ Key only in memory

**Cons:**
- ❌ Lost when terminal closes
- ❌ Need to set each time

---

### Option B: Add to Shell Profile (Persistent)

Add to your `~/.zshrc` or `~/.bashrc`:

```bash
# Add at the end of the file
export ANTHROPIC_API_KEY="sk-ant-api03-..."
```

Then reload:
```bash
source ~/.zshrc  # or source ~/.bashrc
```

**Pros:**
- ✅ Persistent across sessions
- ✅ Available in all terminals

**Cons:**
- ❌ Visible in shell history
- ❌ Shared across all projects

---

### Option C: Use `.env` File (Project-Specific)

1. **Create `.env` file** (already in `.gitignore`):
   ```bash
   cp .env.example .env
   ```

2. **Edit `.env`** and add your key:
   ```bash
   ANTHROPIC_API_KEY=sk-ant-api03-...
   ```

3. **Load in terminal** (before running tests):
   ```bash
   # Load .env file
   export $(cat .env | xargs)
   
   # Or use a tool like direnv
   ```

**Pros:**
- ✅ Project-specific
- ✅ Not committed to git
- ✅ Easy to manage

**Cons:**
- ❌ Need to load before each session
- ❌ File exists on disk (but gitignored)

---

## Method 2: Cursor-Specific Settings

### Option A: Cursor Settings (Workspace)

1. Open Cursor Settings (Cmd/Ctrl + ,)
2. Search for "terminal environment"
3. Add environment variable:
   ```
   ANTHROPIC_API_KEY: sk-ant-api03-...
   ```

**Note**: This may vary by Cursor version. Check Cursor's documentation for the exact location.

---

### Option B: Cursor Tasks/Launch Configuration

Create `.vscode/settings.json` (already in `.gitignore`):

```json
{
  "terminal.integrated.env.osx": {
    "ANTHROPIC_API_KEY": "sk-ant-api03-..."
  },
  "terminal.integrated.env.linux": {
    "ANTHROPIC_API_KEY": "sk-ant-api03-..."
  },
  "terminal.integrated.env.windows": {
    "ANTHROPIC_API_KEY": "sk-ant-api03-..."
  }
}
```

**⚠️ Warning**: This file is in `.gitignore`, but be careful not to commit it!

---

### Option C: Cursor's Secret Management (If Available)

Some IDEs have built-in secret management. Check Cursor's documentation for:
- Secret storage features
- Encrypted local storage
- Integration with system keychain

---

## Method 3: System Keychain (Most Secure)

### macOS

```bash
# Store in macOS Keychain
security add-generic-password \
  -a "ANTHROPIC_API_KEY" \
  -s "converge-tests" \
  -w "sk-ant-api03-..."

# Retrieve and use
export ANTHROPIC_API_KEY=$(security find-generic-password -a "ANTHROPIC_API_KEY" -s "converge-tests" -w)
```

### Linux (using secret-tool)

```bash
# Store
secret-tool store --label="Anthropic API Key" anthropic-api-key

# Retrieve
export ANTHROPIC_API_KEY=$(secret-tool lookup anthropic-api-key)
```

---

## Method 4: Using `direnv` (Recommended for Development)

1. **Install direnv**:
   ```bash
   # macOS
   brew install direnv
   
   # Linux
   sudo apt install direnv
   ```

2. **Add to shell** (`~/.zshrc` or `~/.bashrc`):
   ```bash
   eval "$(direnv hook zsh)"  # or bash
   ```

3. **Create `.envrc`** in project root:
   ```bash
   dotenv .env
   ```

4. **Allow direnv**:
   ```bash
   direnv allow
   ```

**Pros:**
- ✅ Automatically loads `.env` when entering directory
- ✅ Automatically unloads when leaving
- ✅ Works with gitignored files

---

## Verification

Test that your key is set:

```bash
# Check if variable is set (should show your key)
echo $ANTHROPIC_API_KEY

# Or test with a simple command
cargo test --test integration_prompt_formatting test_format_correctness_edn_vs_plain -- --ignored --nocapture
```

---

## Recommended Setup for Cursor

**Best approach for Cursor development:**

1. **Create `.env` file** (already gitignored):
   ```bash
   cp .env.example .env
   # Edit .env and add your key
   ```

2. **Use direnv** (if available) or manually load:
   ```bash
   export $(cat .env | xargs)
   ```

3. **Run tests**:
   ```bash
   cargo test --test integration_prompt_formatting -- --ignored
   ```

4. **Verify `.env` is gitignored**:
   ```bash
   git status
   # .env should NOT appear
   ```

---

## Troubleshooting

### "ANTHROPIC_API_KEY environment variable not set"

**Solution**: The variable isn't available in the test environment.

**Check**:
```bash
echo $ANTHROPIC_API_KEY
```

**Fix**:
- Set the variable in the same terminal where you run tests
- Or use one of the methods above to make it persistent

### Key works in terminal but not in Cursor

**Solution**: Cursor may use a different environment.

**Fix**:
- Use Cursor's settings to set environment variables
- Or use `.vscode/settings.json` (gitignored)
- Or restart Cursor after setting in terminal

### Key appears in git status

**Solution**: `.env` file isn't gitignored.

**Fix**:
1. Check `.gitignore` includes `.env`
2. If already committed, remove from git:
   ```bash
   git rm --cached .env
   git commit -m "Remove .env from git"
   ```

---

## Security Checklist

Before committing code:

- [ ] `.env` is in `.gitignore`
- [ ] No API keys in source code
- [ ] No API keys in commit history (use `git log -S "sk-ant"` to check)
- [ ] `.env.example` exists (without real keys)
- [ ] Team knows not to commit keys

---

## Getting Your API Keys

### Anthropic (Claude)
1. Go to [Anthropic Console](https://console.anthropic.com/settings/keys)
2. Create a new API key
3. Copy the key (starts with `sk-ant-api03-...`)

### OpenAI (GPT-4, GPT-3.5)
1. Go to [OpenAI Platform](https://platform.openai.com/api-keys)
2. Create a new API key
3. Copy the key (starts with `sk-...`)

### Google Gemini
1. Go to [Google AI Studio](https://makersuite.google.com/app/apikey)
2. Create a new API key
3. Copy the key (starts with `AIza...`)

### Perplexity AI
1. Go to [Perplexity Settings](https://www.perplexity.ai/settings/api)
2. Create a new API key
3. Copy the key (starts with `pplx-...`)

### Qwen (Alibaba Cloud)
1. Go to [DashScope Console](https://dashscope.console.aliyun.com/)
2. Create a new API key
3. Copy the key (format varies)

### OpenRouter (Multi-Provider)
1. Go to [OpenRouter Keys](https://openrouter.ai/keys)
2. Create a new API key
3. Copy the key (starts with `sk-or-v1-...`)
4. Note: One key works for multiple providers via OpenRouter

### MinMax AI
1. Go to [MinMax Platform](https://platform.minmax.ai/)
2. Create a new API key
3. Copy the key (format varies)

### Grok (xAI)
1. Go to [xAI Console](https://console.x.ai/)
2. Create a new API key
3. Copy the key (starts with `xai-...`)

See `.env.example` for all available provider keys.

---

## Rotating Keys

If a key is accidentally exposed:

1. **Immediately revoke** the key in Anthropic Console
2. **Generate a new key**
3. **Update** your local `.env` or environment
4. **Check git history** for any commits with the key
5. **Consider** using git-secrets or similar tools

---

## References

- [Anthropic API Keys](https://console.anthropic.com/settings/keys)
- [direnv Documentation](https://direnv.net/)
- [Git Secrets Best Practices](https://git-secret.io/)

