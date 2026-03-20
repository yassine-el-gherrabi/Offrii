#!/usr/bin/env bash
# Offrii server initialization for Hetzner CX32
# Run as root on a fresh Ubuntu 24.04 server:
#   curl -sSL https://raw.githubusercontent.com/yassinelechef/offrii/main/infra/scripts/setup-server.sh | bash
set -euo pipefail

DEPLOY_USER="deploy"
APP_DIR="/opt/offrii"

echo "=== Offrii Server Setup ==="

# ── System updates ─────────────────────────────────────
echo "[1/7] Updating system..."
apt-get update -qq
DEBIAN_FRONTEND=noninteractive apt-get upgrade -y -qq

# ── Install Docker + Compose ──────────────────────────
echo "[2/7] Installing Docker..."
if ! command -v docker &>/dev/null; then
  curl -fsSL https://get.docker.com | sh
  systemctl enable docker
  systemctl start docker
else
  echo "Docker already installed."
fi

# Verify compose plugin
docker compose version || {
  echo "ERROR: docker compose plugin not found."
  exit 1
}

# ── Create deploy user ────────────────────────────────
echo "[3/7] Creating deploy user..."
if ! id "$DEPLOY_USER" &>/dev/null; then
  useradd -m -s /bin/bash -G docker "$DEPLOY_USER"
  mkdir -p /home/$DEPLOY_USER/.ssh
  chmod 700 /home/$DEPLOY_USER/.ssh

  # Copy root authorized_keys if present (Hetzner adds SSH key on provision)
  if [ -f /root/.ssh/authorized_keys ]; then
    cp /root/.ssh/authorized_keys /home/$DEPLOY_USER/.ssh/authorized_keys
  fi

  chown -R $DEPLOY_USER:$DEPLOY_USER /home/$DEPLOY_USER/.ssh
  chmod 600 /home/$DEPLOY_USER/.ssh/authorized_keys 2>/dev/null || true
  echo "User '$DEPLOY_USER' created and added to docker group."
else
  usermod -aG docker "$DEPLOY_USER" 2>/dev/null || true
  echo "User '$DEPLOY_USER' already exists."
fi

# ── Create app directory ──────────────────────────────
echo "[4/7] Setting up application directory..."
mkdir -p "$APP_DIR"/{secrets,backups}
chown -R $DEPLOY_USER:$DEPLOY_USER "$APP_DIR"
chmod 700 "$APP_DIR/secrets"
echo "Created $APP_DIR with secrets and backups directories."

# ── Firewall (UFW) ───────────────────────────────────
echo "[5/7] Configuring firewall..."
apt-get install -y -qq ufw
ufw default deny incoming
ufw default allow outgoing
ufw allow 22/tcp comment "SSH"
ufw allow 80/tcp comment "HTTP"
ufw allow 443/tcp comment "HTTPS"
ufw --force enable
echo "UFW enabled: SSH(22), HTTP(80), HTTPS(443)"

# ── Fail2Ban ──────────────────────────────────────────
echo "[6/7] Installing fail2ban..."
apt-get install -y -qq fail2ban
cat > /etc/fail2ban/jail.local <<'JAIL'
[DEFAULT]
bantime  = 1h
findtime = 10m
maxretry = 5

[sshd]
enabled = true
port    = ssh
filter  = sshd
logpath = /var/log/auth.log
maxretry = 3
JAIL
systemctl enable fail2ban
systemctl restart fail2ban
echo "fail2ban configured (SSH: 3 attempts, 1h ban)."

# ── Unattended upgrades ──────────────────────────────
echo "[7/7] Enabling unattended upgrades..."
apt-get install -y -qq unattended-upgrades
cat > /etc/apt/apt.conf.d/20auto-upgrades <<'AUTOUPGRADE'
APT::Periodic::Update-Package-Lists "1";
APT::Periodic::Unattended-Upgrade "1";
APT::Periodic::AutocleanInterval "7";
AUTOUPGRADE
echo "Unattended security upgrades enabled."

echo ""
echo "============================================"
echo "  Server setup complete."
echo "============================================"
echo ""
echo "Next steps:"
echo "  1. Add your deploy SSH public key:"
echo "     echo 'ssh-ed25519 ...' >> /home/$DEPLOY_USER/.ssh/authorized_keys"
echo ""
echo "  2. Copy secret files to $APP_DIR/secrets/:"
echo "     - jwt_private.pem"
echo "     - jwt_public.pem"
echo "     - AuthKey.p8"
echo ""
echo "  3. Create .env file at $APP_DIR/.env with production secrets"
echo ""
echo "  4. Clone repo and deploy:"
echo "     su - $DEPLOY_USER"
echo "     cd $APP_DIR"
echo "     git clone https://github.com/yassinelechef/offrii.git ."
echo "     docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d"
echo ""
echo "  5. Set up daily backups:"
echo "     crontab -e  # as deploy user"
echo "     0 3 * * * $APP_DIR/infra/scripts/backup.sh >> /var/log/offrii-backup.log 2>&1"
echo ""
echo "  6. Add GitHub Actions secrets:"
echo "     - HETZNER_HOST: $(curl -4 -s ifconfig.me || echo '<server-ip>')"
echo "     - DEPLOY_SSH_KEY: contents of deploy user's private key"
echo ""
