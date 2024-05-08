## Installation
You can download the automatically cross-compiled installers for the latest release from [__github.com/aarkue/ocedeclare/releases/latest__](https://github.com/aarkue/ocedeclare/releases/latest).

The following installer formats are available:
- `[...].AppImage` for Linux (__Recommended for Linux__)
- `[...]-setup.exe` for Windows (__Recommended for Windows__)
- `[...].dmg` for macOS (__Recommended for macOS__)
- `[...].deb` for Linux (Debian)
- `[...].msi` for Windows
- `[...].app.tar.gz` for macOS

### Docker

Alternatively, you can also easily build and run the project locally using Docker.
This will start a local web server for the backend and the frontend.
Once the container is running, you can open [http://localhost:4567/](http://localhost:4567/) in your browser for the tool frontend.

#### Docker Compose
Run `docker compose up --build` in the project root.


#### Docker Files

- __backend__:
  1. First build using `sudo docker build ./backend -t ocedeclare-backend`
  2. Then run with `docker run --init -p 3000:3000 ocedeclare-backend`
- __frontend__:
  1. First build using `sudo docker build ./frontend -t ocedeclare-frontend`
  2. Then run with `sudo docker run --init -p 4567:4567 ocedeclare-backend`


## Usage
__Note: These instructions might not be up-to-date with the newest version.__

In particular, in the new approach based on binding boxes, the need to manually add object variables globally is eliminated.
To see examples of the new constraints, try out the Auto-Discovery feature and inspect the generated constraint.


### Loading OCELs & Viewing OCEL info
When visiting the initial page of the tool, you can load an OCEL2 file by either choosing a custom OCEL2 file (XML/JSON) or selecting an OCEL2 file from the provided list (only available in the dev/docker setup and only if there are .json/.xml files in the `backend/data` folder).
If the OCEL was loaded successfully, you will automatically be forwarded to the OCEL info view.

<img src="https://github.com/aarkue/ocedeclare/assets/20766652/54ae7aa3-36d5-4d6e-8bf4-66eb1b30d34a" width="50%"/>

<img src="https://github.com/aarkue/ocedeclare/assets/20766652/148beb72-dcbd-4e97-a5e1-2793d2194987" width="50%"/>

### Adding and Evaluating Constraints
Visit the Constraints page using the `Constraints` button in the menu (on the left).
Click the `Add Constraint` button on the top. A boxes with hints will guide you through the creation process.

<img src="https://github.com/aarkue/ocedeclare/assets/20766652/fb31cbfa-8d1d-4cc5-9e61-9ab82109b9a9" width="50%"/>


In particular, the steps are as follows:
1. Add one or more _object variables_. Object variables have a unique _variable name_ (e.g., `or_0`) and an _object type_ (e.g., `objects`). ![image](https://github.com/aarkue/ocedeclare/assets/20766652/279f77c1-4710-4264-b3f8-79920d6736e3)
2. Add one or more _event filter nodes_. You need to select an event type for the node (e.g., `pay order`). ![image](https://github.com/aarkue/ocedeclare/assets/20766652/bf7123f4-f820-41bb-9f01-1da3a2d96d9f)
3. Link the created _node_ with the _object variable_. For that, select the object variable name from the dropdown on the bottom of the node ((1) and (2)). The selectable E2O qualifiers are automatically extracted from the event log. Next, the allowed event count can be configured (e.g., by selecting `1 - 1` for (3)) ![image](https://github.com/aarkue/ocedeclare/assets/20766652/32227b43-81e8-482b-ad08-4604bc4d4acb)
4. Evaluate the constraint using the play button on the top right (1). After the evaluation finished, the number of violations (absolute and percentage) will be shown with the corresponding node (2). ![image](https://github.com/aarkue/ocedeclare/assets/20766652/d291b7e7-ff21-426c-893e-8c1df4c49bd7)

  

### Automatically Discovering Constraints
Constraints can also automatically be discovered using the `Auto-Discovery` button.
You can configure the different types of constraints to discover, as well as the object types for which to discover constraints.

The discovered Constraints are automatically added to the list of constraints and can be manually edited or deleted.

### Saving Constraints
The save button on the top center-right saves the created constraints to local storage in your browser. Saved data will persist on reloads.
Make sure to load the correct OCEL file before evaluating or editing saved constraints.

