"""Store server-only model settings. The API key is never a command argument.

Interactive use: python3 -B tools/configure-demo-model.py
Automation may pass the three-field JSON via stdin with --stdin.
"""
import argparse
import getpass
import json
import os
from pathlib import Path

W = Path(__file__).resolve().parents[1]

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--stdin',action='store_true')
    parser.add_argument('--replace',action='store_true')
    args=parser.parse_args()
    if args.stdin:
        import sys
        try:config=json.loads(sys.stdin.read(8193))
        except (ValueError,UnicodeError):raise SystemExit('Invalid configuration JSON; values suppressed') from None
    else:
        base=input('Base URL [https://chat.cordapp.org/v1]: ').strip() or 'https://chat.cordapp.org/v1'
        model=input('Model [deepseek-flash]: ').strip() or 'deepseek-flash'
        key=getpass.getpass('API key (not displayed): ')
        config=dict(base_url=base,default_model=model,api_key=key)
    if not isinstance(config,dict) or set(config)!={'base_url','default_model','api_key'} or any(not isinstance(v,str) or not v or len(v)>4096 or any(ord(c)<32 for c in v) for v in config.values()):
        raise SystemExit('Invalid configuration fields; values suppressed')
    private=W/'target/private'
    if not private.resolve().is_relative_to(W.resolve()):raise SystemExit('Private directory must remain inside this workspace')
    private.mkdir(mode=0o700,parents=True,exist_ok=True)
    if private.is_symlink():raise SystemExit('Private directory must not be a symlink')
    private.chmod(0o700)
    target=private/'model.json'
    if target.exists() and not args.replace:raise SystemExit('Configuration already exists; use --replace only for an intended rotation')
    temporary=private/f'.model-{os.getpid()}.tmp'
    fd=os.open(temporary,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    try:
        with os.fdopen(fd,'w',encoding='utf-8') as out:json.dump(config,out,ensure_ascii=False);out.write('\n')
        os.replace(temporary,target)
    finally:
        if temporary.exists():temporary.unlink()
    print('Server model configuration saved with mode 0600. Credential not displayed.')

if __name__=='__main__':main()
