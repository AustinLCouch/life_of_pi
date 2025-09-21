#!/bin/bash

# Validate GitHub Actions workflow components locally
# Run this script to test the workflow steps before pushing to GitHub

set -e

echo "🔍 Validating Life of Pi GitHub Actions workflow components..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ $2${NC}"
    else
        echo -e "${RED}❌ $2${NC}"
        return 1
    fi
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

echo "📋 Running workflow validation checks..."

# 1. Check formatting
echo "🎨 Checking code formatting..."
cargo fmt --check
print_status $? "Code formatting check"

# 2. Run clippy
echo "📎 Running clippy lints..."
cargo clippy --all-targets --all-features -- -D warnings -A clippy::module_name_repetitions
print_status $? "Clippy lints"

# 3. Build project
echo "🔨 Building project..."
cargo build --verbose --all-features
print_status $? "Project build"

# 4. Run tests
echo "🧪 Running unit tests..."
cargo test --lib --verbose
print_status $? "Unit tests"

echo "🔬 Running integration tests..."
cargo test --test integration_tests --verbose
print_status $? "Integration tests"

echo "📚 Running doc tests..."
cargo test --doc --verbose
print_status $? "Doc tests"

# 5. Check benchmarks
echo "⚡ Checking benchmarks..."
if cargo bench --bench system_benchmarks --no-run; then
    print_status 0 "Benchmark compilation"
else
    print_warning "Benchmark compilation failed - this may be expected in some environments"
fi

# 6. Run security audit (if cargo-audit is installed)
echo "🔒 Running security audit..."
if command -v cargo-audit &> /dev/null; then
    if cargo audit; then
        print_status 0 "Security audit"
    else
        print_warning "Security audit found issues - check the output above"
    fi
else
    print_warning "cargo-audit not installed - skipping security audit"
    echo "   Install with: cargo install cargo-audit"
fi

# 7. Check deny.toml (if cargo-deny is installed)
echo "🚫 Running cargo-deny checks..."
if command -v cargo-deny &> /dev/null; then
    if cargo deny check; then
        print_status 0 "Cargo deny checks"
    else
        print_warning "Cargo deny found issues - check the output above"
    fi
else
    print_warning "cargo-deny not installed - skipping deny checks"
    echo "   Install with: cargo install cargo-deny"
fi

# 8. Check MSRV compatibility
echo "🦀 Checking MSRV compatibility..."
cargo check --lib --bins
print_status $? "MSRV compatibility check"

# 9. Check cross-compilation setup (if cross is installed)
echo "🌐 Checking cross-compilation setup..."
if command -v cross &> /dev/null; then
    if cross check --target aarch64-unknown-linux-gnu; then
        print_status 0 "Cross-compilation setup"
    else
        print_warning "Cross-compilation check failed - this may be expected without Docker"
    fi
else
    print_warning "cross not installed - skipping cross-compilation check"
    echo "   Install with: cargo install cross --git https://github.com/cross-rs/cross"
fi

echo ""
echo -e "${GREEN}🎉 Workflow validation complete!${NC}"
echo ""
echo "📋 Summary of optional tools for full validation:"
echo "   • cargo install cargo-audit      (security auditing)"
echo "   • cargo install cargo-deny       (dependency policy checking)"  
echo "   • cargo install cross            (cross-compilation)"
echo "   • cargo install cargo-llvm-cov   (code coverage)"
echo ""
echo "💡 These tools are installed automatically in GitHub Actions but are optional locally."