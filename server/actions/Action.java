public interface Action {
    public boolean matchCommand(String command);
    public boolean matchState(String command, Player player, List<Player> player, World world, StrinBuilder error);
    public StringBuilder run(String command, Player player, List<Player> player, World world);
}