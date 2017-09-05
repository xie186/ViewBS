
my %hash;

$hash{"CG"}->{"XX"} ->{"BB"} = 1;

my %test = %{$hash{"CG"}};
print $test{XX}->{"BB"};
