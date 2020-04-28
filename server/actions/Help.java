package server.actions;

import java.util.List;
import java.util.Scanner;

import server.main.World;
import server.objects.Player;
import server.objects.Player.ReadOnlyPlayer;

public class Help implements Action {
    private String help;

    @Override
    public boolean matchCommand(String command) {
        return command.startsWith("help");
    }

    @Override
    public boolean parseCommand(String command, ReadOnlyPlayer player, List<ReadOnlyPlayer> players, World world,
            StringBuilder error) {
        Scanner scan = new Scanner(command);
        scan.next();
        if(!scan.hasNext()) {
            help = null;
        } else {
            help = scan.next();
        }
        scan.close();
        return true;
    }

    @Override
    public StringBuilder run(Player player, List<Player> players, World world) {
        StringBuilder out = new StringBuilder();
        if(help == null) {
            out.append("welcome to the help menu!\n");
            out.append("type 'help action' to learn more about stuff you can do!\n");
            out.append("type 'help stat' to learn more about stats!\n");
            out.append("or type 'help <x>' and i'll try to guess what you want to know about!\n");
        } else if(help.equals("action")) {
            out.append("here's a list of all the actions, and what they are used for:\n");
            for (Action a : Actions.actions) {
                String fullActionName = a.getClass().getName();
                int index = fullActionName.lastIndexOf('.');
                String actionName = fullActionName.substring(index == -1 ? 0 : index + 1, fullActionName.length());
                out.append("\n");
                out.append(actionName + ":\n");
                out.append(a.description());
                out.append("\n");
            }
        } else if(help.equals("stat")) {
            out.append("Base Stats vs. Stats:\n");
            out.append("base stats --> the maximum values that your stats can attain.\n");
            out.append("stats --> the actual current value of your stats.\n");
            out.append("for example, your health under 'stats' might be 7, but your health under 'base stats' might be 10.");
            out.append(" that means your current health is 7, and your max health is 10.\n");
            out.append("Stat Descriptions:\n");
            out.append("health --> health (duh). If this reaches 0, you die, your inventory is cleared, and you are respawned.\n");
            out.append("speed --> determines how many units you can move per turn, and whether you go first/how many turns you or your opponent takes in a battle.\n");
            out.append("dmg --> the base damage you can deal per turn in a battle.\n");
            out.append("view --> the distance that you can see. If you increase view, your 'disp' command will show a larger area.\n");
        } else {
            out.append("did you mean: \n");
            for (Action a : Actions.actions) {
                String fullActionName = a.getClass().getName();
                int index = fullActionName.lastIndexOf('.');
                String actionName = fullActionName.substring(index == -1 ? 0 : index + 1, fullActionName.length());
                if(actionName.toLowerCase().contains(help)) {
                    out.append("\n");
                    out.append(actionName + ":\n");
                    out.append(a.description());
                    out.append("\n");
                }
            }
        }
        return out;
    }

    @Override
    public String description() {
        return "the help command.\n" +
               "usage: help";
    }

    private boolean arrayContains(String[] arr, String s) {
        for(String a : arr) {
            if((a == null && s == null) || (a != null && a.equals(s))) {
                return true;
            }
        }
        return false;
    }
}