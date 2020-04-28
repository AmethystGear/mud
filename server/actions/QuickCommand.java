package server.actions;

import java.util.List;
import java.util.HashMap;
import java.util.Scanner;

import server.main.World;
import server.objects.Player;
import server.objects.Player.ReadOnlyPlayer;
import server.utils.ScannerUtils;

public class QuickCommand implements Action {

    String shortCommand;
    String mappedCommand;
    Action action;

    private static HashMap<String, String> commandMap = new HashMap<String, String>();
    private static HashMap<String, Action> actionMap = new HashMap<String, Action>();

    @Override
    public boolean matchCommand(String command) {
        if(command.startsWith("quick")) {
            return true;
        }
        return commandMap.keySet().contains(command);
    }

    @Override
    public boolean parseCommand(String command, ReadOnlyPlayer player, List<ReadOnlyPlayer> players, World world,
            StringBuilder error) {
        if(!command.startsWith("quick")) {
            action = actionMap.get(command);
            mappedCommand = commandMap.get(command);
            return action.parseCommand(mappedCommand, player, players, world, error);
        } else {
            Scanner scan = new Scanner(command);
            scan.next();
            if(!scan.hasNext()) {
                scan.close();
                error.append("you need to specify the short version of the command!");
                return false;
            }
            shortCommand = scan.next();
            for(Action a : Actions.actions) {
                if(a.matchCommand(shortCommand)) {
                    scan.close();
                    String fullActionName = a.getClass().getName();
                    int index = fullActionName.lastIndexOf('.');
                    String actionName = fullActionName.substring(index == -1 ? 0 : index + 1, fullActionName.length());
                    error.append("your short command maps to " + actionName + " please choose a different name for your short command.");
                    return false;
                }
            }
            if(!scan.hasNext()) {
                scan.close();
                error.append("you need to specify what the short version of the command maps to!");
                return false;
            }
            mappedCommand = ScannerUtils.getRemainingInputAsString(scan);
            scan.close();

            for(Action a : Actions.actions) {
                if(a.matchCommand(mappedCommand)) {
                    try {
                        action = a.getClass().getConstructor().newInstance();
                    } catch (Exception e) {
                        // we should never be in this state. If we are, there is a bug.
                        e.printStackTrace();
                        System.out.println("could not make new instance of action!");
                        return false;
                    }
                    return true;
                }
            }
            error.append("no command matches " + mappedCommand);
            return false;
        }
    }

    @Override
    public StringBuilder run(Player player, List<Player> players, World world) {
        if(shortCommand != null) {
            commandMap.put(shortCommand, mappedCommand);
            actionMap.put(shortCommand, action);
            return new StringBuilder("mapped " + shortCommand + " to " + mappedCommand + " successfully.");
        } else {
            return action.run(player, players, world);
        }
    }

    @Override
    public String description() {
        return "allows mapping of shorter commands to longer commands.\n" +
               "for example, quick 1 descr \"pugenum\" maps '1' to 'descr \"pugenum\"'.\n'" +
               "that means when you type 1, the game will run the command 'descr \"pugenum\"'\n" +
               "this is handy for typing long commands quickly.\n" +
               "usage: quick <shortcut name> <command> (note that your shortcut name cannot have any spaces, but the command you are mapping to can have spaces)\n";
    }

}