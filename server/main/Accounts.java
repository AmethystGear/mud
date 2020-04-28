package server.main;

import java.util.Scanner;
import java.util.HashMap;

import java.io.PrintWriter;

import server.objects.Stats;
import server.objects.Stats.ReadOnlyStats;
import server.objects.Position;
import server.objects.Position.ReadOnlyPosition;
import server.objects.Player;
import server.objects.Int;
import server.objects.Int.ReadOnlyInt;

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
        Account newAcc = new Account(name, player.getInventory(), player.getBaseStats(), player.getPosn(), player.xp());
        accounts.put(name, newAcc);
    }

    public static void login(String name, Player player) {
        Account playerAccount = getAccount(name);
        player.login(name, playerAccount.getInventory(), playerAccount.getStats(), playerAccount.getPosn(), playerAccount.getXP());
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
        private ReadOnlyInt xp;

        public Account(Scanner scan) {
            String nextLine = scan.nextLine();
            while(nextLine.equals("")) {
                nextLine = scan.nextLine();
            }
            name = nextLine;
            Scanner lineScan = new Scanner(scan.nextLine());
            int x = lineScan.nextInt();
            int y = lineScan.nextInt();
            int xpNum = lineScan.nextInt();
            posn = new ReadOnlyPosition(new Position(x, y));
            xp = new ReadOnlyInt(new Int(xpNum));
            inventory = new ReadOnlyStats(new Stats(scan));
            scan.nextLine();
            stats = new ReadOnlyStats(new Stats(scan));
        }

        public Account(String name, ReadOnlyStats inventory, ReadOnlyStats stats, ReadOnlyPosition posn, ReadOnlyInt xp) {
            this.name = name;
            this.inventory = inventory;
            this.stats = stats;
            this.posn = posn;
            this.xp = xp;
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

        public ReadOnlyInt getXP() {
            return xp;
        }

        public void save(PrintWriter file) {
            file.write(name + "\n");
            file.write(posn.x() + " " + posn.y() + " " + xp.get() + "\n");
            inventory.saveTo(file);
            stats.saveTo(file);
            file.write("\n");
        }
    }
}