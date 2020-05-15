package server.main;
import java.io.DataOutputStream;
import java.io.IOException;

public class PlayerOutput {
    private DataOutputStream o;
    public static final int MAX_PACKET_SIZE = 10000; // maximum number of characters we can send at a time.

    public PlayerOutput(DataOutputStream o) {
        this.o = o;
    }

    public void send(StringBuilder s) throws IOException {
        send(s.toString());
    }

    public void send(String s) throws IOException {
        String remainder = s;
        while(remainder.length() > MAX_PACKET_SIZE) {
            String current = remainder.substring(0, MAX_PACKET_SIZE);
            remainder = remainder.substring(MAX_PACKET_SIZE, remainder.length());
            sendPacket(current);
        }
        sendPacket(remainder);
    }

    private void sendPacket(String s) throws IOException {
        if(s.length() > MAX_PACKET_SIZE) {
            throw new IllegalArgumentException("String cannot be greater than " + MAX_PACKET_SIZE + " in length!");
        }
        StringBuilder out = new StringBuilder("\n/begin/\n");
        out.append(s);
        out.append("\n/end/\n");
        o.writeUTF(out.toString());
    }
}
