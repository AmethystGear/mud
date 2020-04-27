package server.main;

import java.util.Scanner;
import java.util.HashMap;

import java.io.PrintWriter;

import server.objects.Stats;
import server.objects.Stats.ReadOnlyStats;
import server.objects.Position;
import server.objects.Position.ReadOnlyPosition;
import server.objects.Player;

public class Accounts {
    private static HashMap<String, Account> accounts = new HashMap<String, Account>();

    public static void load(Scanner file) {
        boolean failedToCreateAccount = false;
        while(!failedToCreateAccount) {
            try {
                Account acc = new Account(file);
                accounts.put(acc.getName(), acc);
            } catch (Exception e) {
                failedToCreateAccount = true;
            }
        }
    }

    public static void save(PrintWriter file) {
        for(Account e : accounts.values()) {
            e.save(file);
        }
        file.close();
    }

    public static void createAccount(String name, Player player) {
        if(accounts.keySet().contains(name)) {
            throw new IllegalArgumentException();
        }
        Account newAcc = new Account(name, player.getInventory(), player.getBaseStats(), player.getPosn());
        accounts.put(name, newAcc);
    }

    public static void login(String name, Player player) {
        Account playerAccount = getAccount(name);
        player.login(name, playerAccount.getInventory(), playerAccount.getStats(), playerAccount.getPosn());
        accounts.remove(name);
        createAccount(name, player);
    }

    private static Account getAccount(String name) {
        if(!accounts.keySet().contains(name)) {
            throw new IllegalArgumentException();
        }
        return accounts.get(name);
    }

    public static class Account {
        private String name;
        private ReadOnlyPosition posn;
        private ReadOnlyStats inventory;
        private ReadOnlyStats stats;

        public Account(Scanner scan) {
            String nextLine = scan.nextLine();
            while(nextLine.equals("")) {
                nextLine = scan.nextLine();
            }
            name = nextLine;
            Scanner lineScan = new Scanner(scan.nextLine());
            int x = lineScan.nextInt();
            int y = lineScan.nextInt();
            posn = new ReadOnlyPosition(new Position(x, y));
            inventory = new ReadOnlyStats(new Stats(scan));
            scan.nextLine();
            stats = new ReadOnlyStats(new Stats(scan));
        }

        public Account(String name, ReadOnlyStats inventory, ReadOnlyStats stats, ReadOnlyPosition posn) {
            this.name = name;
            this.inventory = inventory;
            this.stats = stats;
            this.posn = posn;
        }

        public String getName() {
            return name;
        }

        public ReadOnlyStats getInventory() {
            return inventory;
        }

        public ReadOnlyStats getStats() {
            return stats;
        }

        public ReadOnlyPosition getPosn() {
            return posn;
        }

        public void save(PrintWriter file) {
            file.write(name + "\n");
            file.write(posn.x() + " " + posn.y() + "\n");
            inventory.saveTo(file);
            stats.saveTo(file);
            file.write("\n");
        }
    }
}