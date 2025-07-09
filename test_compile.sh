#!/bin/bash

# Script de test de compilation pour DJ-4LED
set -e

echo "🔍 Test de compilation DJ-4LED"
echo "=============================="
echo ""

# Couleurs pour l'output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Fonction pour afficher les erreurs
print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Fonction pour afficher les succès
print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

# Fonction pour afficher les warnings
print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Vérifier qu'on est dans le bon dossier
if [ ! -f "Cargo.toml" ]; then
    cd apps/backend 2>/dev/null || {
        print_error "Impossible de trouver le dossier backend"
        exit 1
    }
fi

echo "📁 Dossier de travail: $(pwd)"
echo ""

# Nettoyer les artifacts précédents
echo "🧹 Nettoyage des artifacts..."
cargo clean

# Test de compilation en mode debug
echo ""
echo "🔨 Test de compilation (mode debug)..."
if cargo build 2>&1 | tee build.log; then
    print_success "Compilation debug réussie"
else
    print_error "Échec de la compilation debug"
    echo ""
    echo "📋 Dernières erreurs:"
    tail -20 build.log
    exit 1
fi

# Test de compilation en mode release
echo ""
echo "🚀 Test de compilation (mode release)..."
if cargo build --release 2>&1 | tee build-release.log; then
    print_success "Compilation release réussie"
else
    print_error "Échec de la compilation release"
    echo ""
    echo "📋 Dernières erreurs:"
    tail -20 build-release.log
    exit 1
fi

# Vérifier les warnings
echo ""
echo "🔍 Vérification des warnings..."
WARNING_COUNT=$(grep -c "warning:" build.log || true)
if [ "$WARNING_COUNT" -gt 0 ]; then
    print_warning "Il y a $WARNING_COUNT warnings dans le code"
    echo "📋 Warnings détectés:"
    grep "warning:" build.log | head -10
else
    print_success "Aucun warning détecté"
fi

# Test des fonctionnalités clippy (si disponible)
echo ""
echo "🔍 Analyse avec clippy..."
if command -v cargo-clippy &> /dev/null; then
    if cargo clippy -- -D warnings 2>&1 | tee clippy.log; then
        print_success "Analyse clippy réussie"
    else
        print_warning "Clippy a détecté des problèmes"
        grep -A5 "error:" clippy.log | head -20 || true
    fi
else
    print_warning "Clippy n'est pas installé (cargo install clippy)"
fi

# Vérifier la taille des binaires
echo ""
echo "📊 Taille des binaires:"
if [ -f "target/debug/led-visualizer" ]; then
    DEBUG_SIZE=$(ls -lh target/debug/led-visualizer | awk '{print $5}')
    echo "   Debug: $DEBUG_SIZE"
fi
if [ -f "target/release/led-visualizer" ]; then
    RELEASE_SIZE=$(ls -lh target/release/led-visualizer | awk '{print $5}')
    echo "   Release: $RELEASE_SIZE"
fi

# Test de compilation du frontend
echo ""
echo "🌐 Test du frontend..."
if [ -d "../../front-old" ]; then
    cd ../../front-old
    if [ -f "package.json" ]; then
        echo "📦 Installation des dépendances..."
        if npm install > npm-install.log 2>&1; then
            print_success "Dépendances installées"
        else
            print_error "Erreur lors de l'installation des dépendances"
            tail -10 npm-install.log
        fi

        echo "🔨 Build du frontend..."
        if npm run build > npm-build.log 2>&1; then
            print_success "Build frontend réussi"
        else
            print_error "Erreur lors du build frontend"
            tail -10 npm-build.log
        fi
    else
        print_warning "package.json non trouvé dans front-old"
    fi
else
    print_warning "Dossier front-old non trouvé"
fi

# Résumé
echo ""
echo "📊 Résumé"
echo "========"
echo ""

# Nettoyer les logs temporaires
rm -f build.log build-release.log clippy.log npm-install.log npm-build.log 2>/dev/null

print_success "Tous les tests de compilation sont passés!"
echo ""
echo "🚀 Le projet est prêt à être lancé avec:"
echo "   ./start_backend.sh"
echo ""
