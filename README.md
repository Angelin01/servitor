# servitor
A "terrible idea" Rest API to control systemd services

One liner to generate token:
```shell
pip install --user argon2-cffi
echo "token" | python -c 'import sys;from argon2 import PasswordHasher;p=PasswordHasher();print(p.hash(sys.stdin.readline().strip()))'
```
