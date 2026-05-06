# Infrastructure as Code Skill

---

name: infrastructure-as-code
description: Implements infrastructure as code using Terraform and Ansible for TraderX. Use when provisioning cloud resources, configuring servers, or automating infrastructure setup.

---

## When to Activate

Use when:
- Provisioning cloud resources (AWS, GCP, Azure)
- Configuring servers and workstations
- Automating infrastructure setup
- Managing infrastructure with version control
- Implementing infrastructure reproducibility

## Core Principles

### Everything as Code
All infrastructure must be codified:
- No manual server configuration
- No manual cloud resource provisioning
- All changes go through version control
- All changes are auditable and reversible

### Idempotency
Infrastructure code must be idempotent:
- Running it multiple times yields same result
- No side effects from re-running
- Safe to re-apply without destroying existing resources

### State Management
- Use Terraform state files to track infrastructure
- Store state in secure, versioned location (S3, Terraform Cloud)
- Lock state to prevent concurrent modifications
- Backup state regularly

## Implementation Checklist

- [ ] Identify infrastructure to codify
- [ ] Choose tool (Terraform for cloud, Ansible for servers)
- [ ] Write Terraform modules or Ansible playbooks
- [ ] Test in development environment
- [ ] Add state management
- [ ] Add secrets management (Vault, environment variables)
- [ ] Document infrastructure decisions
- [ ] Review and audit infrastructure code

## Common Pitfalls

- ❌ Hardcoding secrets → use Vault or environment variables
- ❌ Manual infrastructure changes → always go through IaC
- ❌ No state backup → backup Terraform state regularly
- ❌ No testing → test in dev before prod
- ❌ No documentation → document infrastructure decisions

## TraderX-Specific Adaptations

### Terraform for Cloud Resources
```hcl
# terraform/main.tf
provider "aws" {
  region = "us-east-1"
}

resource "aws_instance" "traderx_server" {
  ami           = "ami-0c55b159cbfafe1f0"
  instance_type = "c5.2xlarge"  # High network performance
  tags = {
    Name = "traderx-hft-server"
  }
}

resource "aws_ebs_volume" "nvme_pool" {
  availability_zone = "us-east-1a"
  size              = 1000  # 1TB NVMe for BAM grids
  type              = "gp3"
  iops              = 16000
  throughput       = 500
}
```

### Ansible for Server Configuration
```yaml
# ansible/playbooks/setup-hft-server.yml
---
- name: Setup HFT Trading Server
  hosts: hft_servers
  become: yes
  tasks:
    - name: Install Rust toolchain
      shell: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    
    - name: Install Python dependencies
      apt:
        name:
          - python3-pip
          - python3-venv
          - python3-dev
    
    - name: Configure kernel for low latency
      sysctl:
        name: "{{ item.name }}"
        value: "{{ item.value }}"
        state: present
      with_items:
        - { name: "net.core.rmem_max", value: "134217728" }
        - { name: "net.core.wmem_max", value: "134217728" }
        - { name: "net.ipv4.tcp_low_latency", value: "1" }
```

### Secrets Management
```hcl
# terraform/secrets.tf
resource "aws_secretsmanager_secret" "traderx_api_keys" {
  name = "traderx/api-keys"
}

resource "aws_secretsmanager_secret_version" "traderx_api_keys_version" {
  secret_id = aws_secretsmanager_secret.traderx_api_keys.id
  secret_string = jsonencode({
    binance_api_key = var.binance_api_key
    binance_secret_key = var.binance_secret_key
  })
}
```

## Verification

After implementing IaC:
- [ ] Infrastructure code is idempotent
- [ ] Secrets are managed securely (not in code)
- [ ] State is backed up and versioned
- [ ] Infrastructure tested in dev environment
- [ ] Documentation explains infrastructure decisions
- [ ] Changes are auditable and reversible
