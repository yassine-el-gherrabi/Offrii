# Offrii Server Setup — Hetzner CX32

**Server**: Hetzner Cloud CX32 (4 vCPU, 8GB RAM, 80GB NVMe)
**IP**: <SERVER_IP>
**OS**: Ubuntu 24.04 LTS
**Location**: Falkenstein, Germany
**Created**: 2026-03-20

---

## Step 1: First SSH Connection (as root)

```bash
ssh root@<SERVER_IP>
```

## Step 2: Create deploy user

```bash
# Create user with home directory and bash shell
adduser --disabled-password --gecos "Offrii Deploy" deploy

# Add to sudo group (for admin commands)
usermod -aG sudo deploy

# Allow sudo without password (needed for automated deploys)
echo "deploy ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/deploy

# Copy SSH key from root to deploy user
mkdir -p /home/deploy/.ssh
cp /root/.ssh/authorized_keys /home/deploy/.ssh/authorized_keys
chown -R deploy:deploy /home/deploy/.ssh
chmod 700 /home/deploy/.ssh
chmod 600 /home/deploy/.ssh/authorized_keys
```

## Step 3: Harden SSH

```bash
# Edit SSH config
sed -i 's/^#\?PermitRootLogin.*/PermitRootLogin no/' /etc/ssh/sshd_config
sed -i 's/^#\?PasswordAuthentication.*/PasswordAuthentication no/' /etc/ssh/sshd_config
sed -i 's/^#\?PubkeyAuthentication.*/PubkeyAuthentication yes/' /etc/ssh/sshd_config

# Restart SSH
systemctl restart sshd
```

After this, root login is disabled. Only `deploy` user with SSH key can connect.

## Step 4: Firewall (UFW)

```bash
ufw default deny incoming
ufw default allow outgoing
ufw allow 22/tcp    # SSH
ufw allow 80/tcp    # HTTP (Caddy redirect to HTTPS)
ufw allow 443/tcp   # HTTPS (Caddy)
ufw --force enable
```

## Step 5: Fail2ban

```bash
apt-get install -y fail2ban

cat > /etc/fail2ban/jail.local << 'EOF'
[sshd]
enabled = true
port = ssh
filter = sshd
maxretry = 3
bantime = 3600
findtime = 600
EOF

systemctl enable fail2ban
systemctl start fail2ban
```

## Step 6: Automatic Security Updates

```bash
apt-get install -y unattended-upgrades
dpkg-reconfigure -plow unattended-upgrades
```

## Step 7: Install Docker

```bash
# Docker official install
curl -fsSL https://get.docker.com | sh

# Add deploy user to docker group (no sudo needed for docker)
usermod -aG docker deploy

# Verify
docker --version
docker compose version
```

## Step 8: Create App Structure

```bash
mkdir -p /opt/offrii/{secrets,backups}
chown -R deploy:deploy /opt/offrii
```

## Step 9: Verify

```bash
# Test deploy user SSH (from local machine)
ssh deploy@<SERVER_IP>

# Test docker
docker run --rm hello-world

# Check firewall
ufw status

# Check fail2ban
fail2ban-client status sshd
```

---

## Setup Completed ✅

| Step | Status |
|------|--------|
| deploy user created | ✅ |
| Root login disabled | ✅ |
| Password auth disabled | ✅ |
| UFW firewall (22/80/443) | ✅ |
| Fail2ban (3 attempts → 1h ban) | ✅ |
| Unattended security upgrades | ✅ |
| Docker 29.3.0 + Compose 5.1.1 | ✅ |
| /opt/offrii structure | ✅ |

---

## Post-Setup: DNS Records

Add A records at your DNS provider:
- `api.offrii.com` → <SERVER_IP>
- `staging.offrii.com` → <SERVER_IP>
- `grafana.offrii.com` → <SERVER_IP>

## Post-Setup: Copy Secrets

```bash
# From local machine
scp .env deploy@<SERVER_IP>:/opt/offrii/.env
scp jwt-keys/private.pem deploy@<SERVER_IP>:/opt/offrii/secrets/jwt_private.pem
scp jwt-keys/public.pem deploy@<SERVER_IP>:/opt/offrii/secrets/jwt_public.pem
scp apns-keys/AuthKey.p8 deploy@<SERVER_IP>:/opt/offrii/secrets/apns_key.p8
```

## Post-Setup: Deploy

```bash
ssh deploy@<SERVER_IP>
cd /opt/offrii
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```
