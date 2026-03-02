
# Creates invite code
INVITE_CODE_JSON="$(curl --request POST --user admin:admin-pass http://localhost:2583/xrpc/com.atproto.server.createInviteCode --header "Content-Type: application/json" --data '{"useCount":1}')"
INVITE_CODE="$(echo "$INVITE_CODE_JSON" | sed 's/{"code":"//g; s/"}//g')"
read -p "Email for the account: " email
read -p "Handle for the account (e.g., example.com, example.test or anything ending in .test is preferred): " handle
read -p "Password for the account: " password
curl --request POST \
    http://localhost:2583/xrpc/com.atproto.server.createAccount \
    --header "Content-Type: application/json" \
    --data "{\"email\":\"$email\",\"handle\":\"$handle\",\"password\":\"$password\",\"inviteCode\":\"$INVITE_CODE\"}"