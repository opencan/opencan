# OpenCAN

OpenCAN is CAN for you and me.

CAN is nothing new and is not that hard to understand. It happens to be
extremely useful for communication in realtime distributed systems, which is
why it's still popular today despite being a decades-old technology.

Large companies exist to make and market tools to help use CAN, from firmware
frameworks to desktop tools to analyze CAN logs. This software is useful,
but like so many other tools in the CAN world, both closed- and open-source,
it has a heavy focus on interfacing with existing (mostly automotive) industry
standards, like CANopen (yes, similar name to OpenCAN!) and J1939, as well
as generally idiosyncratic standard practice.

CAN is inexpensive, powerful, and should be simple. OpenCAN is there to make
that happen. Our goals:

- Be free and open-source.
- Make CAN as easy to possible for you to use for your projects. This means
  OpenCAN primarily focuses on making new CAN systems, rather than interacting
  with or reverse engineering existing ones.
- Choose simplicity over breadth of features.
- Coalesce open-source CAN efforts behind a new network format description
  to replace the closed-source DBC standard.

## CAN Messages

We call computers (microcontrollers, laptops, ...) connected to each other 
with CAN ***nodes*** on a ***CAN network***. Especially when talking about the
physical connection, people also refer to this as a ***CAN bus***.

Communication on CAN is broadcast - when a ***message*** is being sent by some
node on the bus, all nodes can see it.

Every message is composed of two important parts - the ***message id*** and the
***data***. There are 0-8 bytes of data in a CAN message.

## Signals

The message data typically contains more than one piece of information.

Let's start thinking in terms of an example: say we want to broadcast both the
temperature inside and outside a car using CAN. If each temperature measurement
fits in 1 byte, we could put both of them in a single two-byte message, and
decide that the first byte is, say, the temperature outside, and the second
byte is the temperature inside.

```
       data
   2 bytes wide
 <--------------->
|        |        |
| byte 1 | byte 2 |
 -----------------
 ^^^^^^^^---------- signal A: outside temperature
          ^^^^^^^^---------- signal B: inside temperature
```

We call each of these divided parts of the message data field ***signals***.

How does some other node know how and which signals are laid out in the message
once it recieves it? Easy: remember the message id? Ahead-of-time, we decide
on the layout of all the potential messages (which signals are in which
messages, etc), and assign each layout a message id.

Then, at runtime, when a node recieves a message (id + data) over the bus, it can
tell how to interpret its data (signals) based on the id and ahead-of-time
knowledge of how to interpret all the kinds of messages.

!!! note
    Now is a good time to note that 'message' really means two different
    but related things. One usage of 'message' refers to an actual
    ***CAN frame***, the unit of transmission of CAN, which contains id, data,
    and other fields, recieved over the physical bus at runtime.

    The other usage of 'message' refers to a predefined id + arrangement
    of signals. Both of these usages are common in the field, and like most,
    OpenCAN uses them interchangeably.

## Naming and Conventions

These are the naming conventions that OpenCAN follows.

### Nodes and Messages

Each node on the network generally has a name and sends messages that 'belong'
to it. Every message has an owning node and can only be sent by that node.

While this is not a hardware limitation of CAN, it is a nearly universal
practice and is generally how you want to do it. OpenCAN only supports this
scheme and does not support sending the same message from multiple nodes.

### Nodes
Every node has a unique uppercase name like `BRAKE` or `THROTTLE`.

### Messages
Every message has a unique corresponding name. The name is unique across the
network and exactly corresponds to a message id and layout.

Message names start with the node they come from, followed by an underscore,
and then an UpperCamelCase descriptive name, like `BRAKE_CaliperTemperatures`.

### Signals
Every signal also has a unique corresponding name, unique across the entire
network.

Signal names start with the node they come from, followed by an underscore,
and then a lowerCamelCase descriptive name, like `BRAKE_leftFrontCaliperTemp`.

Signal names do not include the names of the message they come from and are
unique across the *network*, not just the message. For more information, see
(link).


