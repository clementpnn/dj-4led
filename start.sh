#!/bin/bash

# DJ-4LED - Version Production Only

echo "🏭 DJ-4LED - PRODUCTION MODE"
echo "==========================="

# Couleurs
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

# Variables
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo -e "${CYAN}Architecture: Tauri (Production)${NC}"
echo -e "${CYAN}Frontend: Vue.js${NC}"
echo -e "${CYAN}Backend: Rust${NC}"

# FORCER LE MODE PRODUCTION
export LED_MODE="production"
export TAURI_LED_MODE="production"

echo -e "${GREEN}🏭 MODE PRODUCTION FORCÉ${NC}"
echo -e "${YELLOW}Variables LED:${NC}"
echo "   LED_MODE=$LED_MODE"
echo "   TAURI_LED_MODE=$TAURI_LED_MODE"

# Vérifier les dépendances essentielles
echo -e "${CYAN}Vérification dépendances...${NC}"

if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Cargo (Rust) requis${NC}"
    exit 1
fi

if ! command -v pnpm &> /dev/null; then
    echo -e "${RED}❌ pnpm requis${NC}"
    exit 1
fi

# Installation dépendances si nécessaire
if [ ! -d "node_modules" ]; then
    echo -e "${CYAN}Installation dépendances frontend...${NC}"
    pnpm install
fi

cd src-tauri
if [ ! -d "target" ]; then
    echo -e "${CYAN}Build dépendances Rust...${NC}"
    cargo check
fi
cd "$SCRIPT_DIR"

# Nettoyage
cleanup() {
    echo ""
    echo -e "${CYAN}🛑 Arrêt...${NC}"
    pkill -f "tauri dev" 2>/dev/null
    pkill -f "cargo run" 2>/dev/null
    echo -e "${GREEN}✅ Arrêt terminé${NC}"
    exit 0
}

trap cleanup INT TERM

# Démarrage
echo ""
echo -e "${CYAN}======================================${NC}"
echo -e "${CYAN}🚀 DÉMARRAGE DJ-4LED PRODUCTION${NC}"
echo -e "${CYAN}======================================${NC}"

echo -e "${YELLOW}🔧 Variables finales:${NC}"
echo "   LED_MODE=$LED_MODE"
echo "   TAURI_LED_MODE=$TAURI_LED_MODE"

echo -e "${BLUE}🚀 Démarrage Tauri en mode production...${NC}"

# Démarrer avec variables d'environnement
LED_MODE="production" TAURI_LED_MODE="production" pnpm tauri dev

echo -e "${GREEN}🎉 DJ-4LED PRODUCTION TERMINÉ${NC}"
