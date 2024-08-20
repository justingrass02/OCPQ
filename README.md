## Installation
You can download the automatically cross-compiled installers for the latest release from [__github.com/aarkue/ocedeclare/releases/latest__](https://github.com/aarkue/ocedeclare/releases/latest).

The following installer formats are available:
- `[...].AppImage` for Linux (__Recommended for Linux__)
- `[...].msi` for Windows (__Recommended for Windows__)
- `[...].dmg` for macOS (__Recommended for macOS__)
- `[...].deb` for Linux (Debian)
- `[...]-setup.exe` for Windows
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

### Loading OCELs
![image](https://github.com/user-attachments/assets/98210a69-cd3d-4c75-a2c5-0ee5f2e44d94)

### Constraint Overview
![image](https://github.com/user-attachments/assets/1604911e-e06e-4099-bcb8-628d89b2a4bd)

### Constraint Editor
#### Adding Nodes
A new node can be added using the button on the top right of the editor.
![image](https://github.com/user-attachments/assets/a1d19f6a-2bf0-4a9d-85aa-f65db7e35f8c)

### Adding object and event variables
Inside the newly created node, object and event variables, as well as filter predicates can be added using the corresponding `+`-buttons inside the node.
In this example, we first create an object variable `o1` with the object type `orders`.
The variable name (1) and type (2) can be selected from a list of available values and the new variable added using the button (3).

![image](https://github.com/user-attachments/assets/05106376-8094-44d3-bc1a-0dc5dbd4c152)
![image](https://github.com/user-attachments/assets/336d7bd3-c986-4f4e-a714-c2af0696b3fe)

Similiarly, we also add an event variable `e1` of type `confirm order`.

![image](https://github.com/user-attachments/assets/b7b5286e-3b65-4318-827b-91204964b4eb)
![image](https://github.com/user-attachments/assets/6f46c0f5-c9a4-4904-aadc-52c0afa32ea4)

The updated node then looks as shown below, indicating the added variables and their types.

![image](https://github.com/user-attachments/assets/268fefb5-1a55-4e43-9ac3-7c47e7317557)

### Adding Filter Predicates

Next, we want to add a predicate statement linking `o1` and `e1`.
For that, we add a new filter predicate using the `+`-button shown besides the filters.
Then, the filter type (_Object associated with Event_) can be selected (1), and the corresponding parameters (2), (3) and (4) can be configured.


![image](https://github.com/user-attachments/assets/4af9e8ee-8a15-497b-a4cb-6e2aeaf15a46)

Finally, using the _Add_ button (5) the filter predicate is added to the node, which then looks as shown below.

![image](https://github.com/user-attachments/assets/9ca87d3c-2bb9-47c7-b628-3352bba0fa71)

### Evaluating Queries and Constraints

Constructed constraints and queries can be evaluated using the play button at the top right (1).
After the evaluation finishes, the evaluation results are shown directly inside the editor at the corresponding nodes (2).
For instance, the query constructed so far yields 2000 results (i.e., 2000 output bindings).
As there are no constraint predicates for the node, no violation percentage is shown.


![image](https://github.com/user-attachments/assets/9f311c29-a872-4860-80a8-90df679372df)

### Adding Child Nodes

Next, we add a child node by first creating a new node (using the corresponding button on the top right) and then connect both nodes using the connection handles on the nodes (1) and (2).

![image](https://github.com/user-attachments/assets/14a714f5-920a-4e6a-aab1-147c2c7ebf92)

Clicking on the `-` button of the connection edge allows assigning a name to this edge. In this example, we name the edge `A`.
![image](https://github.com/user-attachments/assets/74070713-259c-4c69-b03f-3b582108a2c6)

### Adding Constraints

Using this added child node, we want to specify a constraint regarding the number of child bindings (i.e., the number of `pay order` events for the placed order `o1`).
This can be done by first clicking the `+` button next to the constraints of the node and then selecting the _Number of Child Bindings_ constraint type, and configuring the associated paramters (specifying the edge name `A` as well as the min and max count, both `1` in this example).

![image](https://github.com/user-attachments/assets/de200cf7-1af0-41b1-a1bb-20c38a0a896f)


After adding this constraint and evaluating it (again using the play button on the top right), we can see that this constraint is satisfied for all bindings (i.e, a violation percentage of 0% is shown, and the node is colored in bright green).

![image](https://github.com/user-attachments/assets/c615ea0f-9227-436d-95d6-cb0c2e2c3ecc)


Finally, we want to make this constraint a little more interesting.
In particular, we want to specify that the `pay order` event should occur within 2 weeks after the `place order` event.
For this, we add a filter predicate to the child node, such that it only queries `pay order` events within this timeframe.
![image](https://github.com/user-attachments/assets/aa834fcd-e4b3-4e87-9262-74fd9d73710d)

Evaluating this updated constraint again yields a violation percentage of 29.3%.

![image](https://github.com/user-attachments/assets/21e46af5-676c-4fc3-8b02-8f0a98ef9035)


An alternative way to model this constraint in this specific setting would be adding this time between event predicate as a constraint to the child node.
Note, that this constraint might be slightly different in general, as it simply requires that _all `pay order`_ events fulfill this constraint. 
In this case the constraint can also be modeled using just one node, as shown below.

![image](https://github.com/user-attachments/assets/d3753c87-a95f-4538-a001-b91fe791705b)


## Old Usage
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

