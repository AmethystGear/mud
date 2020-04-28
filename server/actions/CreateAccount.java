package server.actions;

import java.util.List;
import java.util.Scanner;

import server.main.World;
import server.objects.Player;
import server.objects.Player.ReadOnlyPlayer;
import server.main.Accounts;

public class CreateAccount implements Action {
    String name;

    @Override
    public boolean matchCommand(String command) {
        return command.startsWith("createAccount");
    }

    @Override
    public boolean parseCommand(String command, ReadOnlyPlayer player, List<ReadOnlyPlayer> players, World world,
            StringBuilder error) {
        Scanner scan = new Scanner(command);
        scan.next();
        if(!scan.hasNext()) {
            error.append("you have to type in your name to make a new account.");
            scan.close();
            return false;
        }
        name = scan.next();
        scan.close();
        return true;
    }

    @Override
    public StringBuilder run(Player player, List<Player> players, World world) {
        try {
            Accounts.createAccount(name, player);
            return new StringBuilder("created account successfully.");
        } catch(IllegalArgumentException e) {
            return new StringBuilder("Account creation failed: there is already a user with that name! " +
                                     "Use login <name> to login to an account with that name on this server.\n");
        }
    }

    @Override
    public String description() {
        return "creates a new account with the provided name.\n" +
               "usage: createAccount <name>\n";
    }

}