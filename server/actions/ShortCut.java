package server.actions;

import java.util.List;
import java.util.HashMap;
import java.util.Scanner;

import server.main.World;
import server.objects.Player;
import server.objects.Player.ReadOnlyPlayer;
import server.utils.ScannerUtils;

public class ShortCut implements Action {

    private static HashMap<Integer, HashMap<String, String>> commandMap = new HashMap<>();
    private static HashMap<Integer, HashMap<String, Action>> actionMap = new HashMap<>();

    private String shortCommand;
    private String mappedCommand;
    private Action action;

    @Override
    public boolean matchCommand(String command, int playerID) {
        if(command.startsWith("short")) {
            return true;
        }
        if(!commandMap.keySet().contains(playerID)) {
            commandMap.put(playerID, new HashMap<>());
            actionMap.put(playerID, new HashMap<>());
        }
        return commandMap.get(playerID).keySet().contains(command);
    }

    @Override
    public boolean parseCommand(String command, ReadOnlyPlayer player, List<ReadOnlyPlayer> players, World world,
            StringBuilder error) {

        if(!command.startsWith("short")) {
            action = actionMap.get(player.ID()).get(command);
            mappedCommand = commandMap.get(player.ID()).get(command);
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
            if(commandMap.get(player.ID()).keySet().contains(shortCommand)) {
                commandMap.get(player.ID()).remove(shortCommand);
                actionMap.get(player.ID()).remove(shortCommand);
            }
            for(Action a : Actions.actions) {
                if(a.matchCommand(shortCommand, player.ID())) {
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
                if(a.matchCommand(mappedCommand, player.ID())) {
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
            commandMap.get(player.ID()).put(shortCommand, mappedCommand);
            actionMap.get(player.ID()).put(shortCommand, action);
            return new StringBuilder("mapped " + shortCommand + " to " + mappedCommand + " successfully.");
        } else {
            player.lastAction = action;
            player.lastCommand = mappedCommand;
            return action.run(player, players, world);
        }
    }

    @Override
    public String description() {
        return "allows mapping of shorter commands to longer commands.\n" +
               "for example, short 1 descr \"pugenum\" maps '1' to 'descr \"pugenum\"'.\n'" +
               "that means when you type 1, the game will run the command 'descr \"pugenum\"'\n" +
               "this is handy for typing long commands quickly.\n" +
               "usage: short <shortcut name> <command> (note that your shortcut name cannot have any spaces, but the command you are mapping to can have spaces)\n";
    }

}
