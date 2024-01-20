## Building a release
```sh
# install pyinstaller with pip
pip install pyinstaller
# build the standalone executeable
pyinstaller --onefile --noconsole --add-data images:images mouse.py
```