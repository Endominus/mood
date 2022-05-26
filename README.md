# What is `mood`?

`mood` is a tracking and analysis tool for people enacting lifestyle changes. This could mean exercise or meditation routines,  medication or dietary changes, or basically anything else that can be tracked numerically. It also tries to correlate noteworthy events to what you intend to monitor through the use of its tag system.

## How does it work?

When first setting up the software, you will be asked for a number of fields to monitor. These can be outputs (the actual things that you want to change - for example, you might put "mood" or "productivity" in here), inputs (the things that might affect output, such as "exercise" or "overtime" or "social event") or a hybrid between the two (such as "sleep quality"). The software will ask you how you intend to track these fields - either as numerical values (how was your mood out of ten? how many calories did you eat? how many hours of good sleep did you get?) or booleans (did you play with your children? did you take a walk outside?). After that, you just need to run the software every day. Each time you do, it will prompt you for the data for that day, as well as tags you can add.

You also have the ability to specify "states," which are default-true changes that you expect to have longer-term effects. For example, "moved to Boston" could be a state, as could "had knee surgery". This allows you to check if these changes are positively or negatively impacting the fields you wish to track.

### Tags

Tags are meant to indicate rare events that might strongly impact one of your outputs. They can be more general or variable than a field, but function similarly to a boolean field. For example, if you are diligent in reporting them, the software might discover that days that you have the `phone_call:daryl` tag, your mood is usually higher - perhaps you ought to talk to him more often? Or the `ate:japanese` tag is highly correlated with the `indigestion` field being true; you might have some sensitivity to soy that might be the cause of that complaint? Think of tags as catch-all terms for things you think *might* be affecting your mood, but can't confirm/don't want to specify every day.

## Then what?

Obviously, just having the data around doesn't do much. `mood` is also analysis software, meaning that after you have filled in data for long enough (at least a month, though I would encourage waiting longer) it will try to determine if there are any trends in your inputs and outputs and correlate them. It will also try to determine if your boolean inputs may cause anomalous behaviour in numerical outputs, or if changes in states cause different trends. By doing so, you can (hopefully) get some clarity as to the effectiveness of whatever lifestyle change you are enacting.

# How to install

Just download the latest executable from the downloads directory. Run `mood init` from command line; it will create a config file and database in the appropriate location. Then follow the prompts to set up your fields and you'll be good to go!