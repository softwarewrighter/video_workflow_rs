#!/bin/bash
# Check for Python environment hygiene violations
# Run from repo root: ./scripts/check-python-hygiene.sh

set -e

ERRORS=0

echo "Checking for Python hygiene violations..."

# Check 1: No direct pip/pip3 usage in shell scripts
echo -n "  Checking for direct pip usage... "
if grep -rn "pip install" --include="*.sh" --include="*.bash" . 2>/dev/null | grep -v "uv pip" | grep -v check-python-hygiene; then
    echo "FAIL: Found direct pip usage (should use 'uv pip')"
    ERRORS=$((ERRORS + 1))
else
    echo "OK"
fi

# Check 2: No pip in Rust code string literals (except in comments/docs)
echo -n "  Checking for pip in Rust code... "
if grep -rn '"pip install' --include="*.rs" components/ 2>/dev/null; then
    echo "FAIL: Found pip install in Rust code"
    ERRORS=$((ERRORS + 1))
else
    echo "OK"
fi

# Check 3: Workflow YAML files should use python_path variable, not hardcoded python3
echo -n "  Checking workflows use python_path... "
WORKFLOWS_WITHOUT_PYTHON_PATH=$(grep -l "kind: tts_generate\|kind: text_to_image" projects/*/workflow.yaml 2>/dev/null | while read f; do
    if ! grep -q "python_path:" "$f" 2>/dev/null; then
        echo "$f"
    fi
done)
if [ -n "$WORKFLOWS_WITHOUT_PYTHON_PATH" ]; then
    echo "FAIL: These workflows lack python_path:"
    echo "$WORKFLOWS_WITHOUT_PYTHON_PATH"
    ERRORS=$((ERRORS + 1))
else
    echo "OK"
fi

# Check 4: Venv exists
echo -n "  Checking .venv exists... "
if [ ! -d ".venv" ]; then
    echo "FAIL: No .venv directory (run: uv venv)"
    ERRORS=$((ERRORS + 1))
else
    echo "OK"
fi

# Check 5: Required packages installed in venv
echo -n "  Checking required packages in venv... "
if [ -f ".venv/bin/python" ]; then
    if .venv/bin/python -c "import requests; from gradio_client import Client" 2>/dev/null; then
        echo "OK"
    else
        echo "FAIL: Missing packages (run: source .venv/bin/activate && uv pip install -r requirements.txt)"
        ERRORS=$((ERRORS + 1))
    fi
else
    echo "SKIP (no venv)"
fi

echo ""
if [ $ERRORS -gt 0 ]; then
    echo "Found $ERRORS hygiene violation(s)"
    exit 1
else
    echo "All Python hygiene checks passed!"
    exit 0
fi
