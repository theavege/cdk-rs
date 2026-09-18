# Easy Console Interfaces with CDK

*Posted by La rédaction | Author: William Daniau — originally published in Linux Magazine (France), issues 107 and 108*

This article is an introduction to CDK, in which we present, through examples, the most useful widgets so you can quickly build efficient console-mode interfaces.

Even in 2008, there are many situations where you might want to build a console-mode interface — in fact, any time X11 isn't usable. Whether it's for lack of bandwidth (yes, not everyone has a fast connection — my own link doesn't exceed 1 Mbit/s) or simply because X11 isn't available at all, for example on an embedded system or a live rescue system, etc.

To develop console-mode interfaces on Linux there is a library available on every Linux system, called **ncurses**. ncurses is to console interfaces what **xlib** is to graphical interfaces. Everything needed to build a console interface is in ncurses, but it stays fairly low-level. Although there's an *ncurses howto* among the standard howtos, ncurses is still not a library that's very intuitive to approach, and it takes a good deal of time before you can build a decent interface with it.

There is a higher-level, easier-to-approach library that bundles many pre-built *widgets*. It's called CDK, an acronym for *Curses Development Kit* [1]. CDK is a library written in C. It is nonetheless written in a very object-oriented spirit — each widget has methods in the form of associated functions. A rigorous definition of the structures, combined with pointer casting, gives it something resembling inheritance. CDK is very well documented; the man pages are detailed (though they do contain a few copy/paste errors). You should get in the habit of consulting them systematically for each widget. Reading the *headers* (`*.h`) can also be very useful, since the structures are not described in the man pages. Most widgets come with a small example program, but unfortunately these aren't written in a very pedagogical way and aren't trivial to read.

At the following address [2] you'll find a *tarball* containing all the example programs from this article, along with the version of CDK used, which is version 5.0 from May 7, 2006.

To keep the article more "digestible," it is split into two parts. In the first part, we cover the basics of programming with CDK along with a few simple widgets; in the second part, we look at more elaborate widgets, as well as building form pages and drop-down menus.

---

## Part 1: The Basics

### 1. Installation

First, we need to install CDK. Before anything else, check that the `ncurses` package and the `ncurses` development package are installed on your machine.

CDK is included in some distributions (this is the case for OpenSUSE 10.3, for example, though the available version is 4.9). If that's the case, you can install the CDK package and its development package (headers, etc.) with your usual package manager. Otherwise, you'll need to compile it from source, which is a classic `./configure && make && make install` process.

### 2. Hello World

Every CDK program follows the same skeleton:

```c
#include <cdk/cdk.h>
int main() {
CDKSCREEN *cdkscreen;
WINDOW *cursesWin;
...
declarations etc
...
```

We first initialize ncurses with `initscr()`, then initialize a CDK screen from the resulting curses window with `initCDKScreen()`. Every widget is then created by passing it this `cdkscreen` and is attached to it.

You compile this program with the command:

```
gcc hello_world.c -lcdk -lncurses -o hello_world
```

which, on execution, produces the screen shown in Figure 1 (a simple "Hello World" label).

Let's look at the different parts of this program in more detail. A **CDKLabel** widget is created like this:

```c
/* Define the label */
monlabel = newCDKLabel(cdkscreen,
CENTER, /* x coordinate: starting column */
CENTER, /* y coordinate: starting line from the top */
letexte, /* the character array containing the label text */
3, /* number of lines to display */
TRUE, /* draw a box around it? */
FALSE); /* draw a shadow? */
/* Display it */
refreshCDKScreen (cdkscreen);
/* Wait for the user to press a key */
waitCDKLabel(monlabel,' ');
/* clean up before exiting */
destroyCDKLabel (monlabel);
destroyCDKScreen (cdkscreen);
endCDK();
}
```

### 3. Adding Color

CDK also lets you color and style displayed text using special in-band tags inside the character strings, such as:

```c
letexte[1]="";
letexte[2]="</59>Hello World!<!59>";
/* Initialization */
cursesWin = initscr();
cdkscreen = initCDKScreen (cursesWin);
/* Initialize CDK colors */
initCDKColor();
....
....
```

The tag above tells CDK to use color pair number 48 for the first line and color pair number 59 for the third line. Note that you must initialize the color system by calling `initCDKColor()` for this to work. Doing so produces the screen shown in Figure 2 ("Hello World" rendered in color).

The table below (Table 1) lists the most useful of these tags.

| Tag | Effect |
|---|---|
| `</xx>Text<!xx>` | Applies color pair `xx` to the text (`xx` = 1 to 64) |
| `</B>Text<!B>` | Bold text |
| `</U>Text<!U>` | Underlined text |
| `</K>Text<!K>` | Blinking text |
| `</R>Text<!R>` | Reverse-video text |
| `</N>Text<!N>` | Normal text |

*Table 1: The most useful tags*

A **CDKLabel** with more lines and no box can also be created this way:

```c
/* Define the label */
monlabel = newCDKLabel(cdkscreen,
CENTER, /* x coordinate: starting column */
CENTER, /* y coordinate: starting line from the top */
letexte, /* the character array containing the label text */
10, /* number of lines to display */
TRUE, /* draw a box around it? */
FALSE); /* draw a shadow? */
....
}
```

*(see Figure 3, "Still more fun...")*

### 4. Let's Have a Dialogue

A label is nice — it can even look good — but it's a bit lightweight, even for the simplest of interfaces. The first real feature you'll want is the ability to ask the user questions. CDK provides a widget made up of a label plus a fixed set of answer buttons. Most of the time you'll answer yes or no, but you aren't limited to two choices. In the following example we use three choices: yes, no, or grumpy. The resulting screen is shown in Figure 4.

The comments in the example explain the various parameters. The one that needs a bit more explanation is the `highlight` parameter, the eighth argument of the `newCDKDialog` function, which specifies the *ncurses* attribute used for the button currently selected by the user. Table 3 lists the most useful of these attributes. Note that these ncurses attributes can be combined with a binary "or." For example, `A_UNDERLINE|A_BLINK` gives an underlined, blinking character.

| Attribute | Effect |
|---|---|
| `A_NORMAL` | (Nothing special, as the name suggests) |
| `A_UNDERLINE` | Underlined character |
| `A_REVERSE` | Reverse-video character |
| `A_BOLD` | Bold character |
| `A_BLINK` | Blinking character |
| `COLOR_PAIR(n)` | Use color pair `n` |

*Table 3: The most useful ncurses attributes*

*(Figure 4: the CDKDialog dialogue box)*

The new element here is the use of the associated `activate` function — in this case `activateCDKDialog`. This function handles the event loop for us: it decides what to do when the user presses a given key, i.e. switching the active button, validating the entry, etc. Every widget structure has an `int` member called `exitType`, which you can check once the `activate` function returns. This member can take the values shown in Table 4. Note there's another possible value, `vEARLY_EXIT`, which we won't cover in this tutorial. Note also that CDK's standard behavior lets you always exit an `activate` function with the Escape key — it's up to the programmer to override this if it doesn't suit their needs.

| Value | Meaning |
|---|---|
| `vNORMAL` | Normal exit from the widget |
| `vESCAPE_HIT` | Exited via the Escape key |
| `vNEVER_ACTIVATED` | Initial state |

*Table 4: Values of the `exitType` member*

As for retrieving information from a widget, this can be done directly through the `activate` function itself (as in this example), through specific associated functions, or by accessing the structure directly.

This example also introduces two very useful functions, described in `man cdk_util`: `popupLabel` and `popupDialog`. `popupLabel` displays a label centered on the screen and waits for any keypress, after which the label disappears. `popupDialog` displays a dialog centered on the screen and waits for the user's choice (a value of -1 is returned if the user exits with Escape), then the dialog disappears. These two functions bundle together the creation, display, activation, and destruction of an object. The configuration options for these label/dialog helpers are, however, more limited.

Let's now run a small experiment, first noting that in our example, the "question" CDKDialog remains on screen after we've finished with the activation, as you can see in Figure 5 — unlike the *pop-up* versions, which disappear automatically.

Erasing a widget is done with the associated `erase` function; in our case, adding:

```c
...
selection = activateCDKDialog (question, 0);
eraseCDKDialog(question);
/* What we exited with and what was answered */
...
```

will erase the dialog from the screen once we're done using it. But be careful: the widget itself is not destroyed, it is merely erased from the screen. It can be redrawn individually with the associated `draw` function, and it will also be redisplayed on every screen refresh — which notably happens after a pop-up window closes, as you'll see by running this program. If you want it to disappear for good, you need to use the associated `destroy` function, which erases the object and frees all the memory allocated to it. So, if you move the `destroyCDKDialog(question)` call used for final cleanup at the end of the program to where `eraseCDKDialog(question)` currently is, you'll find that the dialog indeed no longer reappears.

### 5. Let's Open Up a Bit

With dialogs, we've seen how to ask the user a closed-ended question. Now let's see how to ask open-ended questions, using the **Entry** family of widgets. This family has three widgets: `CDKEntry` for simple single-line entries, `CDKTemplate` for entries with an input mask, and finally `CDKMentry` for multi-line entries.

#### 5.1 Simple Entry and Callback

*(Figure 5: the CDKDialog dialog after user input)*

Let's first look at a simple-entry example:

```c
#include <cdk/cdk.h>
int saisieCB(EObjectType cdktype, void *object, void
*clientData, chtype key);
int main() {
WINDOW *cursesWin;
CDKENTRY *saisie;
CDKSCREEN *cdkscreen;
void *clientData,
chtype key)
{
return (TRUE);
}
```

The `EObjectType` type is a CDK enum whose possible values are described on the `cdk_binding` man page. It represents the widget's type. In short: when the `saisie` (entry) widget is activated and the user presses the [F1] key, our callback function `saisieCB` is called with the following arguments:

```c
saisieCB(vENTRY,saisie,compteur,KEY_F(1))
```

To finish off this example, inside the `saisieCB` routine we need, in order to display a `popupLabel`, a pointer to the CDK screen (`screen`) of the object that called `saisie`. There are at least three ways to get it.

**Method 1**

CDK provides a macro named `ScreenOf` that returns the `CDKSCREEN` associated with a widget, but you have to give this macro a typed pointer, whereas the function actually receives a `void` pointer. So you need to cast it:

```c
screen = ScreenOf((CDKENTRY *)object);
```

**Method 2**

Every CDK widget structure is built so that its first member, `obj`, is a pointer to a `CDKOBJS` structure. That structure's `screen` member is the one we want. Since `obj` is the first member of the structure, you can cast your pointer directly to `CDKOBJS`:

```c
screen = ((CDKOBJS *)object)->screen;
```

**Method 3**

You can also reach the `obj` member and then the `screen` member directly, though the resulting code starts to look like everything I dislike about C — but to each their own...

```c
screen = (&((CDKENTRY *)object)->obj)->screen;
```

#### 5.2 Multi-line Entry

The `CDKMentry` widget (`man cdk_mentry`) lets you do multi-line user entry. I'll only cover the creation part here, since usage is similar to what we've already seen.

```c
saisie = newCDKMentry(cdkscreen,
CENTER, /* x coordinate: starting column */
CENTER, /* y coordinate: starting line from the top */
titre, /* the entry's title */
label, /* text displayed before the entry field */
A_REVERSE|COLOR_PAIR(24), /* attribute for text typed by
the user */
'_'|COLOR_PAIR(24), /* character used to fill the field */
vMIXED, /* character filter (see man cdk_display) */
40, /* field width. 0 -> widest possible on screen
-N -> widest possible minus N characters */
5, /* field height (same rules as width) */
3, /* number of lines in the field -> a bit odd, in
that the fill character fills all the lines anyway,
so this only serves to limit the maximum entry length */
9, /* minimum entry size, -1 => -1 corresponds to 0 */
TRUE, /* draw a box around it? */
FALSE); /* draw a shadow? */
```

The resulting screen is shown in Figure 7 (multi-line entry).

#### 5.3 Template

The `CDKTemplate` widget (`man cdk_template`) lets you build input masks, as in this example where we ask the user for their date of birth:

```c
#include <cdk/cdk.h>
```

*(continues with the date-of-birth mask example)*

---

## Part 2: Lists, Forms and Menus

We continue our CDK tutorial [1]. In the first part, we saw the basics of programming with CDK, a few basic widgets, and the callback-function mechanism. For now, we're able to build "sequential" interfaces that ask the user one question after another, which is a bit limited. In this part we'll see how to manage complete forms and drop-down menus, letting us build more ergonomic interfaces. But first, we'll start by looking at a new, particularly useful family of widgets.

Let's recall, before we start, that at the following address [2] you'll find a *tarball* containing all the example programs for this article, along with the version of CDK used, which is version 5.0 from May 7, 2006.

### 1. Lists

We'll now look at a very useful family of widgets, which let you choose one or more elements from a list.

#### 1.1 Scrolling List and Radio List

The simplest widget in this family has a somewhat misleading name, since it doesn't contain the word "list." It is the **CDKScroll** widget, which displays a scrolling list of items, optionally shown as a numbered bulleted list; once activated, you navigate with the arrow keys and select with the [Enter] key. The resulting screen is shown in Figure 1.

```c
#include <cdk/cdk.h>
int main() {
CDKSCREEN *cdkscreen;
WINDOW *cursesWin;
CDKSCROLL *liste;
char *titre="</48>Select a vegetable from the list";
char temp[256],*mesg[10];
int choix;
char *leglist[] = {"Carrot","Turnip","Cauliflower",
"Broccoli","Potato","Zucchini","Eggplant",
"Bell pepper","Rutabaga"};
/* Initialization */
cursesWin = initscr();
cdkscreen = initCDKScreen (cursesWin);
/* Initialize CDK colors */
initCDKColor();
liste = newCDKScroll(cdkscreen,
CENTER, /* x coordinate: starting column */
CENTER, /* y coordinate: starting line from the top */
RIGHT, /* scrollbar position (RIGHT,LEFT,NONE) */
8, /* height of the list */
30, /* width of the list */
titre, /* widget title */
leglist, /* the list of items */
9, /* number of items */
FALSE, /* if TRUE, numbered bullet list */
A_REVERSE, /* ncurses attribute for the selected item */
TRUE, /* draw a box around it? */
FALSE); /* draw a shadow around it? */
/* Display it */
refreshCDKScreen (cdkscreen);
while(liste->exitType != vNORMAL) /* We don't exit on Escape */
{
choix = activateCDKScroll (liste, 0);
}
sprintf(temp,"<C>Your selection result: %i",choix);
mesg[0]=copyChar(temp);
sprintf(temp,"<C>%s",leglist[choix]);
mesg[1]=copyChar(temp);
popupLabel(cdkscreen,mesg,2);
/* clean up before exiting */
destroyCDKScroll(liste);
destroyCDKScreen (cdkscreen);
endCDK();
}
```

*(Figure 1: CDKScroll list)*

A variant of this widget is the **CDKRadio** widget. To draw an analogy with graphical interfaces, it's a mix between a drop-down list and radio buttons. Once activated, navigation is done with the arrow keys. You choose the list element with the spacebar, and finalize the selection with the [Enter] key. I'll only cover the widget's creation here, since its usage is exactly the same as `CDKScroll`. The resulting screen is shown in Figure 2.

```c
legumes = newCDKRadio(cdkscreen,
CENTER, /* x coordinate: starting column */
CENTER, /* y coordinate: starting line from the top */
LEFT, /* scrollbar position RIGHT,LEFT,NONE */
9, /* height of the list */
20, /* width of the list */
"<C></48>Choose a vegetable for dinner", /* Title */
leglist, /* array of strings making up the list */
9, /* number of items to display */
'#', /* selection character */
0, /* index of the default item */
A_REVERSE, /* ncurses attribute for the current item */
TRUE, /* draw a box? */
FALSE); /* draw a shadow? */
```

*(Figure 2: CDKRadio list)*

#### 1.2 Alpha List

The **CDKAlphalist** widget is a composite widget made up of a user entry field and an alphabetically sorted drop-down list (hence its name). Once activated, the arrow keys let you move through the list. The current item is reflected in the entry field. If you type a character into the entry field — in our example a "C" — the current item jumps to the first one starting with "C." Pressing [Tab] performs an autocompletion if possible, or otherwise displays a *popup* list of possible matches, as shown in Figure 3.

```c
#include <cdk/cdk.h>
int main() {
CDKSCREEN *cdkscreen;
WINDOW *cursesWin;
```

*(Figure 3: autocompletion in CDKAlphalist)*

#### 1.3 Selection

To close out this family of widgets, the **CDKSelection** widget is, in graphical-interface terms, a mix between a checkbox and a drop-down list. This widget displays a list where each item's tag can be toggled through a certain number of states (in our example `[ ]`, `[Y]`, and `[N]`). This state cycles with the spacebar. The resulting screen is shown in Figure 4. Retrieving the information requires direct access to the structure, as done in this example.

```c
#include <cdk/cdk.h>
int main() {
CDKSCREEN *cdkscreen;
WINDOW *cursesWin;
CDKSELECTION *legumes;
CDKLABEL *tleg;
/*char **itemlist;*/
char *itemlist[30];
char *mesg[10];
char temp[256];
char *choix[] = {"[ ]","[Y]","[N]"};
char *leglist[] = {"Carrot","Turnip","Cauliflower",
"Broccoli","Potato",
"Zucchini","Eggplant","Bell pepper","Rutabaga"};
int i,lcount;
/* Initialization */
cursesWin = initscr();
cdkscreen = initCDKScreen (cursesWin);
/* Initialize CDK colors */
...
{
/*
sprintf(temp,"%s",leglist[i-1]);
mesg[lcount]=copyChar(temp);*/
mesg[lcount]=leglist[i-1];
lcount++;
}
}
popupLabel(cdkscreen,mesg,lcount);
destroyCDKSelection (legumes);
destroyCDKScreen (cdkscreen);
endCDK();
}
```

*(Figure 4: Selection widget)*

### 2. Forms

#### 2.1 A Form

Alright, all of this is nice, but everything we know how to do so far amounts to "sequential" interfaces — one widget after another — which is a bit limited. CDK planned for this with the `traverseCDKScreen` function (`man cdk_traverse`). Let's look at the following example, a screenshot of which is shown in Figure 5 (page 66).

```c
#include <cdk/cdk.h>
/* Declaration of the callback functions */
/* Help */
int helpCB(EObjectType cdktype, void *object, void
*clientData, chtype key);
/* Vegetable help */
int helplegCB(EObjectType cdktype, void *object, void
*clientData, chtype key);
/* Ok button callback */
int okCB(EObjectType cdktype, void *object, void *clientData,
chtype key);
/* Cancel button callback */
```

*(help-callback body)*

```c
CDKSCREEN *screen;
CDKOBJS *obj = (CDKOBJS *)object;
screen=obj->screen;
char *mesg[10];
mesg[0]="<C></48>Vegetables";
mesg[1]="";
mesg[2]="<C>Use the arrow keys to change item";
mesg[3]="<C>and press the spacebar to";
mesg[4]="<C>change the state";
mesg[5]="";
popupLabel(screen,mesg,6);
}
/* CallBack for the Ok button */
/* We only use the object parameter, which lets us
retrieve the cdkscreen */
int okCB(EObjectType cdktype, void *object, void *clientData,
chtype key)
{
CDKOBJS *obj = (CDKOBJS *)object;
CDKSCREEN *screen;
screen=obj->screen;
exitOKCDKScreen (screen);
}
/* CallBack for the Cancel button */
/* We only use the object parameter, which lets us
retrieve the cdkscreen */
int cancelCB(EObjectType cdktype, void *object, void
*clientData, chtype key)
{
CDKOBJS *obj = (CDKOBJS *)object;
CDKSCREEN *screen;
screen=obj->screen;
exitCancelCDKScreen (screen);
}
```

*(Figure 5: A form)*

*(Figure 6: Drop-down menus)*

### 4. Conclusion

That brings us to the end of this short tutorial. You should now be able to build your own console interfaces quickly with this excellent CDK library.

There are still many aspects of CDK we haven't covered — additional widgets such as `CDKCalendar` and `CDKSlider`, as well as other features such as the `lowerCDKObject` and `raiseCDKObject` functions, which let you manage overlapping objects by sending one to the back or bringing it to the front, respectively.

Also very useful is the associated `positionCDKxxxx` function, which lets you move a widget interactively.

Reading the man pages and the *headers* will take care of the rest...

---

**References**

[1] Curses Development Kit: http://invisible-island.net/cdk/

[2] Example files for this tutorial: http://www.femto-st.fr/~daniau/CDK/

*Originally published in Linux Magazine (France), issues 107 and 108.*
