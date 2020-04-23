package server.actions;

import java.util.List;
import java.util.Scanner;

import server.main.World;
import server.objects.Player;
import server.utils.ScannerUtils;

public class Eat implements Action {

    // parameters
    private int numToEat;
    private String item;

    @Override
    public boolean matchCommand(String command) {
        Scanner scan = new Scanner(command);
        boolean result = scan.hasNext() && scan.next().equals("eat");
        scan.close();
        return result;
    }

    @Override
    public boolean parseCommand(String command, Player.ReadOnlyPlayer player, List<Player.ReadOnlyPlayer> players, World world, StringBuilder error) {
        Scanner scan = new Scanner(command);
        scan.next();
        if(!scan.hasNextInt()) {
            scan.close();
            error.append("you need to specify the amount you are going to eat!");
            return false;
        }
        int numToEat = scan.nextInt();
        String item = ScannerUtils.getRemainingInputAsString(scan).replace('-', ' ');
        
        scan.close();
        return true;
    }

    @Override
    public StringBuilder run(Player player, List<Player> players, World world) {
        // TODO Auto-generated method stub
        return null;
    }

}