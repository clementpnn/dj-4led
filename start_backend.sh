#!/bin/bash

# Script simple pour démarrer le backend DJ-4LED

echo "🎵 DJ-4LED Backend"
echo "=================="
echo ""

# Couleurs
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RED='\033[0;31m'
NC='\033[0m'

# Variables
BACKEND_PID=""
FORCE_BUILD=false

# Vérifier les arguments
if [[ "$1" == "--force" || "$1" == "-f" ]]; then
    FORCE_BUILD=true
    echo -e "${YELLOW}🔄 Build forcé activé${NC}"
fi

# Aller dans le bon répertoire
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Vérifier les contrôleurs
echo -e "${CYAN}📡 Vérification des contrôleurs...${NC}"
CONTROLLERS_OK=0
for ip in 192.168.1.45 192.168.1.46 192.168.1.47 192.168.1.48; do
    if ping -c 1 -W 1 $ip > /dev/null 2>&1; then
        ((CONTROLLERS_OK++))
    fi
done

if [ $CONTROLLERS_OK -gt 0 ]; then
    echo -e "${GREEN}✅ $CONTROLLERS_OK/4 contrôleurs détectés${NC}"
    MODE_ARGS="--production"
else
    echo -e "${YELLOW}⚠️  Mode test activé${NC}"
    MODE_ARGS="--production --test"
fi

# Déterminer le répertoire de travail selon la structure
if [ -f "Cargo.toml" ]; then
    # On est déjà dans la racine du projet
    WORK_DIR="."
    echo -e "${CYAN}📂 Projet Rust détecté à la racine${NC}"
elif [ -d "apps/backend" ] && [ -f "apps/backend/Cargo.toml" ]; then
    # Structure avec apps/backend
    WORK_DIR="apps/backend"
    echo -e "${CYAN}📂 Backend: apps/backend${NC}"
elif [ -d "backend" ] && [ -f "backend/Cargo.toml" ]; then
    # Structure avec backend
    WORK_DIR="backend"
    echo -e "${CYAN}📂 Backend: backend${NC}"
else
    echo -e "${RED}❌ Aucun Cargo.toml trouvé${NC}"
    echo -e "${YELLOW}💡 Vérifiez que vous êtes dans le bon répertoire${NC}"
    exit 1
fi

cd "$WORK_DIR"

# Vérifier que c'est le bon projet
if ! grep -q "led-visualizer\|DJ-4LED\|led_visualizer" Cargo.toml 2>/dev/null; then
    echo -e "${YELLOW}⚠️  Le Cargo.toml ne semble pas correspondre au projet led-visualizer${NC}"
fi

# Afficher des infos de debug
echo -e "${CYAN}🔍 Debug info:${NC}"
echo "   Répertoire: $(pwd)"
echo "   Cargo.toml: $([ -f Cargo.toml ] && echo "✅" || echo "❌")"
echo "   src/: $([ -d src ] && echo "✅" || echo "❌")"

# Déterminer le nom du binaire depuis Cargo.toml
BINARY_NAME="led-visualizer"
if [ -f "Cargo.toml" ]; then
    # Essayer de lire le nom du binaire depuis Cargo.toml
    TOML_NAME=$(grep -E "^name\s*=" Cargo.toml | head -1 | sed 's/.*=\s*"\([^"]*\)".*/\1/' | tr '-' '_')
    if [ ! -z "$TOML_NAME" ]; then
        BINARY_NAME="$TOML_NAME"
    fi
fi

echo "   Binaire attendu: target/release/$BINARY_NAME"

# Compilation
echo -e "${CYAN}🔨 Compilation...${NC}"

# Forcer le build si demandé
if [ "$FORCE_BUILD" = true ]; then
    echo "   Nettoyage..."
    cargo clean
fi

# Déterminer si on doit compiler
SHOULD_BUILD=false

if [ "$FORCE_BUILD" = true ]; then
    SHOULD_BUILD=true
    echo "   Raison: build forcé"
elif [ ! -f "target/release/$BINARY_NAME" ]; then
    SHOULD_BUILD=true
    echo "   Raison: binaire manquant"
elif [ "Cargo.toml" -nt "target/release/$BINARY_NAME" ]; then
    SHOULD_BUILD=true
    echo "   Raison: Cargo.toml modifié"
elif [ -d src ] && [ "$(find src -name "*.rs" -newer "target/release/$BINARY_NAME" | head -1)" ]; then
    SHOULD_BUILD=true
    echo "   Raison: sources modifiées"
fi

if [ "$SHOULD_BUILD" = true ]; then
    echo "   Building..."

    # Compilation avec output détaillé en cas d'erreur
    if ! cargo build --release; then
        echo -e "${RED}❌ Erreur compilation${NC}"
        echo -e "${YELLOW}💡 Suggestions:${NC}"
        echo "   • Vérifiez les erreurs Rust ci-dessus"
        echo "   • Tentez: cargo check"
        echo "   • Ou: cargo build --release --verbose"
        exit 1
    fi
    echo -e "${GREEN}✅ Compilé${NC}"
else
    echo -e "${GREEN}✅ Déjà compilé${NC}"
fi

# Vérifier que le binaire existe maintenant
if [ ! -f "target/release/$BINARY_NAME" ]; then
    echo -e "${RED}❌ Binaire non trouvé après compilation: target/release/$BINARY_NAME${NC}"
    echo -e "${YELLOW}💡 Vérifiez le nom dans Cargo.toml${NC}"
    exit 1
fi

# Copier config si elle existe
for config in "../config.vivid.toml" "../../config.vivid.toml" "config.vivid.toml"; do
    if [ -f "$config" ]; then
        cp "$config" config.toml
        echo -e "${GREEN}✅ Config appliquée: $config${NC}"
        break
    fi
done

# Fonction de nettoyage
cleanup() {
    echo ""
    echo -e "${CYAN}🛑 Arrêt...${NC}"
    if [ ! -z "$BACKEND_PID" ]; then
        kill $BACKEND_PID 2>/dev/null
        wait $BACKEND_PID 2>/dev/null
    fi
    exit 0
}

trap cleanup INT TERM

# Démarrer
echo ""
echo -e "${CYAN}🎵 Démarrage backend...${NC}"
echo -e "${CYAN}🔌 WebSocket: ws://localhost:8080${NC}"
echo ""

./target/release/$BINARY_NAME $MODE_ARGS &
BACKEND_PID=$!

# Vérifier le démarrage
sleep 2
if ! kill -0 $BACKEND_PID 2>/dev/null; then
    echo -e "${RED}❌ Échec démarrage${NC}"
    echo -e "${YELLOW}💡 Vérifiez les logs ci-dessus${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Backend démarré !${NC}"
echo "PID: $BACKEND_PID"
echo "Binaire: ./target/release/$BINARY_NAME"
echo "Ctrl+C pour arrêter"
echo ""

if [[ "$MODE_ARGS" == *"--test"* ]]; then
    echo -e "${YELLOW}⚠️  Mode test${NC}"
fi

# Attendre
wait $BACKEND_PID
