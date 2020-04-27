package server.actions;

import java.util.List;
import java.util.Scanner;

import server.main.World;
import server.objects.Player;
import server.objects.Player.ReadOnlyPlayer;
import server.main.Accounts;

public class Login implements Action {
    String name;

    @Override
    public boolean matchCommand(String command) {
        return command.startsWith("login");
    }

    @Override
    public boolean parseCommand(String command, ReadOnlyPlayer player, List<ReadOnlyPlayer> players, World world,
            StringBuilder error) {
        Scanner scan = new Scanner(command);
        scan.next();
        if(!scan.hasNext()) {
            error.append("you have to type in your name to login.");
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
            Accounts.login(name, player);
            return new StringBuilder("Login successful.\n");
        } catch(IllegalArgumentException e) {
            return new StringBuilder("Login failed: there is no user with that name! Use createAccount <name> to make an account with that name on the server.\n");
        }
    }

}