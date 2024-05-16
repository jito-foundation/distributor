#!/usr/bin/env bash
# Originally authoried by Zantetsu from Shinobi Systems


# Function to print out usage info and exit with error
function usage_error ()
{
    echo -n "Usage: check_stake_weights.sh [RPC_URL] "
    echo "<stakeweight_file_to_check>"
    echo -n "  [RPC_URL] defaults to https://api.mainnet-beta.solana.com "
    echo "if not provided"
    exit 1
}

# Check input: if there are more than 2 args, it is an error
if [ -n "$3" ]; then
    usage_error
# Else if there are two args, then the first is the RPC URL and the second is the
# stake weight file to compare against
elif [ -n "$2" ]; then
    RPC_URL=$1
    FILE_TO_CHECK=$2
# Else if there is one arg, then the RPC URL is the default and the arg is the
# stake weight file to compare against
elif [ -n "$1" ]; then
    RPC_URL=https://api.mainnet-beta.solana.com
    FILE_TO_CHECK=$1
# Else error because there must be at least one arg
else
    usage_error
fi    


# This function checks to make sure its single argument is the name of a
# command that is in the user's path; if not it prints an error and exits
function ensure_in_path ()
{
    if ! type $1 >/dev/null 2>/dev/null; then
        echo -n "ERROR: The program '$1' must be in your path for "
        echo "check_stake_weights.sh"
        echo "to function correctly."
        exit 1
    fi
}


# This function fetches the vote account information from the JSON RPC API at
# the RPC URL, in JSON form, and prints it to stdout
function fetch_vote_accounts ()
{
    if ! curl -s $RPC_URL -X POST -H "Content-Type: application/json" -d \
         '{ "jsonrpc": "2.0", "id": 1, "method": "getVoteAccounts" }'; then
        echo "ERROR: curl failed"
        exit 1
    fi
}


# This function reads from stdin the output of the getVoteAccounts JSON RPC
# API, and prints a tabulation in the same form as the stake weight file to
# check (itself being the output of the spl-feature-proposal propose command).
# The order of pubkeys is sorted; which is not guaranteed to be the same as
# that of the spl-feature-proposal command since the order produced by that
# command is not sorted in any specific order.
function tabulate_stake ()
{
    echo "recipient,amount"

    local jq_command=`cat <<'    EOF'
        .result.current[],.result.delinquent[] |
        "\(.nodePubkey),\(.activatedStake)"
    EOF`
    
    jq -r "$jq_command" | sort
}


# Ensure that curl is in the user's path before proceeding
ensure_in_path curl
# Ensure that jq is in the user's path before proceeding
ensure_in_path jq


# Write the tabulated stake out into a temporary file
FETCHED_TMP=`mktemp`
fetch_vote_accounts | tabulate_stake > $FETCHED_TMP

# Strip the "recipient,amount" line out of the file to compare against, sort
# the pubkeys, add the header back in, and write to a temp file
COMPARE_TMP=`mktemp`
(echo "recipient,amount"; cat $FILE_TO_CHECK | \
     grep -v "^recipient,amount" | sort) > $COMPARE_TMP

# If the files differ, print out the diffs
if ! cmp $COMPARE_TMP $FETCHED_TMP >/dev/null 2>/dev/null; then
    echo "ERROR: Stake weights do not match; diff follows"
    diff $COMPARE_TMP $FETCHED_TMP
    rm $COMPARE_TMP $FETCHED_TMP
    exit 1
fi

# If this point is reached, the comparison succeeded so the fetched weights
# were identical to the weights to compare against
echo "Stake weights match"
rm $COMPARE_TMP $FETCHED_TMP