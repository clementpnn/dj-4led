#!/bin/bash

# DJ-4LED - Version Tauri avec reconnexion automatique

echo "🔄 DJ-4LED - Version Tauri"
echo "=========================="

# Couleurs
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

# Variables
TAURI_PID=""
MONITOR_PID=""
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESTART_COUNT=0
MAX_RESTARTS=10

cd "$SCRIPT_DIR"

# Architecture Tauri détectée
echo -e "${CYAN}Architecture: Tauri (Frontend Vue + Backend Rust intégré)${NC}"
echo -e "${CYAN}Frontend: src/ (Vue.js)${NC}"
echo -e "${CYAN}Backend: src-tauri/ (Rust)${NC}"

# Test contrôleurs LED plus robuste
echo -e "${CYAN}Test des contrôleurs LED...${NC}"
CONTROLLERS_OK=0

# Force production mode si demandé
if [[ "$1" == "--force-production" || "$1" == "-p" ]]; then
    echo -e "${YELLOW}🔧 Mode production FORCÉ par argument${NC}"
    CONTROLLERS_OK=1
    export TAURI_LED_MODE="production"
else
    # Test des contrôleurs
    for ip in 192.168.1.45 192.168.1.46 192.168.1.47 192.168.1.48; do
        echo -n "  Teste $ip... "

        # Test spécifique Windows/Git Bash
        if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
            if ping -n 1 -w 1000 $ip >/dev/null 2>&1; then
                echo -e "${GREEN}OK${NC}"
                ((CONTROLLERS_OK++))
            else
                echo -e "${RED}FAIL${NC}"
            fi
        else
            # Linux/Mac
            if ping -c 1 -W 1 $ip >/dev/null 2>&1; then
                echo -e "${GREEN}OK${NC}"
                ((CONTROLLERS_OK++))
            else
                echo -e "${RED}FAIL${NC}"
            fi
        fi
    done
fi

echo ""
if [ $CONTROLLERS_OK -gt 0 ]; then
    echo -e "${GREEN}✅ MODE PRODUCTION ACTIVÉ${NC}"
    export TAURI_LED_MODE="production"
    MODE_NAME="PRODUCTION"
else
    echo -e "${YELLOW}⚠️ MODE SIMULATEUR${NC}"
    echo -e "${CYAN}   Conseil: utilisez './start.sh --force-production' pour forcer${NC}"
    export TAURI_LED_MODE="simulator"
    MODE_NAME="SIMULATEUR"
fi

# Vérifier les dépendances
echo -e "${CYAN}Vérification des dépendances...${NC}"

# Vérifier Rust/Cargo
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Cargo (Rust) non trouvé${NC}"
    echo -e "${YELLOW}   Installez Rust: https://rustup.rs/${NC}"
    exit 1
fi

# Vérifier Node.js/pnpm
if ! command -v pnpm &> /dev/null; then
    echo -e "${RED}❌ pnpm non trouvé${NC}"
    echo -e "${YELLOW}   Installez pnpm: npm install -g pnpm${NC}"
    exit 1
fi

# Vérifier Tauri CLI
if ! command -v cargo tauri &> /dev/null; then
    echo -e "${YELLOW}⚠️ Tauri CLI non trouvé, installation...${NC}"
    cargo install tauri-cli --quiet
fi

# Installation/mise à jour des dépendances
echo -e "${CYAN}Installation des dépendances...${NC}"

# Dépendances frontend
if [ ! -d "node_modules" ] || [ "package.json" -nt "node_modules/.pnpm/registry.npmjs.org" ]; then
    echo "  Installation dépendances frontend..."
    pnpm install --silent
    if [ $? -ne 0 ]; then
        echo -e "${RED}❌ Erreur installation frontend${NC}"
        exit 1
    fi
fi

# Vérifier/build dépendances Rust
echo "  Vérification dépendances Rust..."
cd src-tauri
if [ ! -d "target" ] || [ "Cargo.toml" -nt "target/.rustc_info.json" ]; then
    echo "  Build des dépendances Rust..."
    cargo check --quiet
    if [ $? -ne 0 ]; then
        echo -e "${RED}❌ Erreur dépendances Rust${NC}"
        exit 1
    fi
fi
cd "$SCRIPT_DIR"

# Fonction de démarrage Tauri
start_tauri() {
    echo -e "${BLUE}🚀 Démarrage Tauri (tentative $((RESTART_COUNT + 1)))...${NC}"

    # Variables d'environnement pour le mode LED
    export TAURI_LED_MODE="$TAURI_LED_MODE"

    # Démarrer en mode développement
    if [[ "$2" == "--release" || "$2" == "-r" ]]; then
        echo -e "${CYAN}   Mode: Release (optimisé)${NC}"
        pnpm tauri build --runner cargo &
        TAURI_PID=$!
    else
        echo -e "${CYAN}   Mode: Development (hot-reload)${NC}"
        pnpm tauri dev &
        TAURI_PID=$!
    fi

    # Attendre et vérifier le démarrage
    sleep 8
    if ! kill -0 $TAURI_PID 2>/dev/null; then
        echo -e "${RED}❌ Tauri échec au démarrage${NC}"
        return 1
    fi

    echo -e "${GREEN}✅ Tauri démarré (PID: $TAURI_PID, Mode LED: $MODE_NAME)${NC}"
    return 0
}

# Fonction de monitoring et reconnexion
monitor_processes() {
    local last_status_time=0

    while true; do
        sleep 10
        local current_time=$(date +%s)

        # Vérifier Tauri
        if [ ! -z "$TAURI_PID" ] && ! kill -0 $TAURI_PID 2>/dev/null; then
            echo -e "${RED}🔄 Tauri arrêté - Redémarrage...${NC}"
            RESTART_COUNT=$((RESTART_COUNT + 1))

            if [ $RESTART_COUNT -gt $MAX_RESTARTS ]; then
                echo -e "${RED}❌ Trop de redémarrages ($MAX_RESTARTS), abandon${NC}"
                break
            fi

            start_tauri
            sleep 5
        fi

        # Statistiques périodiques (toutes les minutes)
        if [ $((current_time - last_status_time)) -ge 60 ]; then
            echo -e "${BLUE}💓 Status: Tauri PID $TAURI_PID, Mode: $MODE_NAME, Redémarrages: $RESTART_COUNT${NC}"
            last_status_time=$current_time
        fi
    done
}

# Nettoyage
cleanup() {
    echo ""
    echo -e "${CYAN}🛑 Arrêt en cours...${NC}"

    [ ! -z "$MONITOR_PID" ] && kill $MONITOR_PID 2>/dev/null
    [ ! -z "$TAURI_PID" ] && kill $TAURI_PID 2>/dev/null

    # Attendre un peu
    sleep 3

    # Force kill si nécessaire
    if [ ! -z "$TAURI_PID" ] && kill -0 $TAURI_PID 2>/dev/null; then
        echo -e "${YELLOW}Force kill Tauri...${NC}"
        kill -9 $TAURI_PID 2>/dev/null

        # Tuer également les processus cargo/node associés
        pkill -f "tauri dev" 2>/dev/null
        pkill -f "cargo run" 2>/dev/null
    fi

    echo -e "${GREEN}✅ Arrêt terminé${NC}"
    exit 0
}

trap cleanup INT TERM

# Gestion des arguments
RELEASE_MODE=""
if [[ "$1" == "--release" || "$1" == "-r" || "$2" == "--release" || "$2" == "-r" ]]; then
    RELEASE_MODE="--release"
fi

# Démarrage initial
echo ""
echo -e "${CYAN}======================================${NC}"
echo -e "${CYAN}🚀 DÉMARRAGE DE DJ-4LED (TAURI)${NC}"
echo -e "${CYAN}======================================${NC}"

# Démarrer Tauri
if ! start_tauri "" "$RELEASE_MODE"; then
    echo -e "${RED}❌ Impossible de démarrer Tauri${NC}"
    exit 1
fi

# Démarrer le monitoring
echo -e "${BLUE}🔍 Démarrage du monitoring automatique...${NC}"
monitor_processes &
MONITOR_PID=$!

# Status final
echo ""
echo -e "${GREEN}🎉 DJ-4LED TAURI DÉMARRÉ AVEC SUCCÈS !${NC}"
echo "═══════════════════════════════════════════════"
echo -e "${CYAN}Application:${NC} PID $TAURI_PID"
echo -e "${CYAN}Mode LED:${NC}    $MODE_NAME"
echo -e "${CYAN}Frontend:${NC}    Vue.js (intégré)"
echo -e "${CYAN}Backend:${NC}     Rust (intégré)"
echo -e "${CYAN}Monitor:${NC}     PID $MONITOR_PID (Auto-restart activé)"
echo -e "${CYAN}Interface:${NC}   Application native + webview"
echo ""
echo -e "${YELLOW}💡 Architecture Tauri unifiée${NC}"
echo -e "${YELLOW}💡 Hot-reload activé en mode dev${NC}"
echo -e "${YELLOW}💡 Reconnexion LED automatique${NC}"
echo -e "${YELLOW}💡 Maximum $MAX_RESTARTS redémarrages${NC}"
echo -e "${YELLOW}💡 Ctrl+C pour tout arrêter${NC}"
echo ""
echo -e "${GREEN}🎵 Ready for LED Audio Visualization!${NC}"

# Attendre
wait $MONITOR_PID
