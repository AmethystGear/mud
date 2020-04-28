package server.actions;

// if you create a new action, you must append it to this array.
public class Actions {
    public static final Action[] actions = new Action[] {
        new Attack(),
        new CreateAccount(),
        new DescribeItem(),
        new Display(),
        new Eat(),
        new Give(),
        new Help(),
        new Login(),
        new Move(),
        new Run(),
        new ShowInventory(),
        new ShowMap(),
        new ShowStats(),
        new Trade(),
        new Upgrade(),
        new ShowXP()
    };
}