# System Prompt Management - Testing Guide

## Quick Compilation Check

```bash
# Essential compilation tests
cargo check --workspace
cargo check -p goose -p goose-cli
```

## Basic Functionality Tests

### 1. CLI Commands
```bash
# List prompts (should show defaults)
goose system-prompt list

# Create new prompt
goose system-prompt create "Code Reviewer" \
  --description "Reviews code for best practices" \
  --content "You are an expert code reviewer..."

# Show details
goose system-prompt show "Code Reviewer"

# Set as default
goose system-prompt set-default "Code Reviewer"
```

### 2. Session Integration
```bash
# Use custom prompt in session
goose session --system-prompt "Code Reviewer"

# Use custom prompt with run command
goose run --text "Hello world" --system-prompt "Code Reviewer"
```

### 3. Recipe Integration
Create `test_recipe.yaml`:
```yaml
version: 1.0.0
title: Test Recipe
description: Testing system prompt integration
instructions: Say hello
settings:
  system_prompt_id: Code Reviewer
```

Test:
```bash
goose run --recipe test_recipe.yaml
```

## Expected Behavior

1. **Default Setup**: Should create default prompts automatically
2. **CRUD Operations**: Create, read, update, delete should work
3. **Name/ID Resolution**: Should find prompts by name or ID
4. **CLI Integration**: `--system-prompt` should work in session/run commands
5. **Recipe Integration**: `system_prompt_id` in recipe settings should work
6. **Fallback**: Should fall back to embedded prompts if system prompts fail

## Troubleshooting

### Compilation Errors
- Check `tabled` and `uuid` dependencies in `goose-cli/Cargo.toml`
- Verify all imports resolve correctly
- Check that `APP_STRATEGY` is properly imported

### Runtime Errors
- Check config directory permissions (`~/.config/goose/`)
- Verify YAML serialization/deserialization works
- Test prompt resolution logic (ID vs name)

### Integration Issues
- Verify `SessionSettings` properly includes `system_prompt_id`
- Check recipe parsing includes new settings field
- Test CLI argument handling for `--system-prompt`

## Key Files Modified

- `crates/goose/src/system_prompts.rs` - Core functionality
- `crates/goose-cli/src/commands/system_prompt.rs` - CLI commands
- `crates/goose/src/agents/prompt_manager.rs` - Integration
- `crates/goose/src/recipe/mod.rs` - Recipe support
- `crates/goose-cli/src/cli.rs` - CLI argument handling
- `crates/goose-cli/src/session/builder.rs` - Session integration

## Success Criteria

✅ Code compiles without errors
✅ CLI commands work as expected
✅ System prompts can be created, modified, and used
✅ Integration with sessions and recipes works
✅ Fallback to default prompts works when needed