
![Logo](./logo.png)
#### Save your memory for more important things! 

Clipboard warrior is a simple Terminal tool to make it easy for you to copy and paste commands and text from your clipboard. You can paste commands into a menu and use your cursor keys to traverse them quickly.

Clipboard warrior uses a flat file to store your commands so you can easily copy them between machines.
</br>
#### Commands

**Arrow keys** - move between columns and rows
**P** - Paste the contents of your clipboard into a new row in the selected menu
**C** - Copy the selected command to your clipboard
**D** - Delete a command
**Q** - Quit
**H** - Home (command list)

</br>
#### Add a menu item

Edit the clipboarddb.json file and add a command with a new menu name, once you have saved the file and reload clipboard-warrior the new menu will now exist.

For example:

```bash
[
    {
        "name": "kubectl get all -A",
        "menu": "Kubernetes"
    },
    {
        "name": "qwfe£&qwr11rff",
        "menu": "Keys"
    }
]
```
Will create two menus, 'Kubernetes' and 'Keys' each with one given command.

</br>
#### Installation

**Cargo build --release**

Make sure that the clipboard.json file is in the same directory as the executable 
(--release will create this in target/release/clip)

Run the shell script to install globally, so you can run it with  ```clip```
```bash
bash global-install.sh
```
