import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import * as z from "zod";

import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import {
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { Icons } from "@/components/ui/icons";
import { Input } from "@/components/ui/input";
import { ResponsiveSelect, type ResponsiveSelectOption } from "@wealthfolio/ui";
import { brokerConnectionSchema } from "@/lib/schemas";
import { useAccounts } from "@/hooks/use-accounts";
import { useBrokerConnectionMutations } from "./use-broker-connection-mutations";

const syncIntervalOptions: ResponsiveSelectOption[] = [
  { label: "Manual", value: "Manual" },
  { label: "Daily", value: "Daily" },
  { label: "Weekly", value: "Weekly" },
];

type BrokerConnectionForm = z.infer<typeof brokerConnectionSchema>;

interface ConnectionFormProps {
  defaultValues?: BrokerConnectionForm;
  onSuccess?: () => void;
}

export function ConnectionForm({
  defaultValues,
  onSuccess = () => undefined,
}: ConnectionFormProps) {
  const { createMutation, updateMutation, testMutation } = useBrokerConnectionMutations({
    onSuccess,
  });
  const { accounts } = useAccounts(false); // Don't filter inactive accounts

  const form = useForm<BrokerConnectionForm>({
    resolver: zodResolver(brokerConnectionSchema),
    defaultValues: defaultValues || {
      name: "",
      accountId: null,
      queryId: "",
      token: "",
      syncInterval: "Manual",
      autoImport: true,
    },
  });

  function onSubmit(data: BrokerConnectionForm) {
    const { id, accountId, ...rest } = data;

    if (id) {
      // Update existing connection
      updateMutation.mutate({
        connectionId: id,
        request: {
          name: rest.name,
          accountId: accountId || null,
          queryId: rest.queryId,
          token: rest.token || undefined, // Only send token if provided
          syncInterval: rest.syncInterval,
          autoImport: rest.autoImport,
        },
      });
    } else {
      // Create new connection
      createMutation.mutate({
        brokerType: "IBKR",
        name: rest.name,
        accountId: accountId || null,
        queryId: rest.queryId,
        token: rest.token,
        syncInterval: rest.syncInterval,
        autoImport: rest.autoImport,
      });
    }
  }

  function handleTestConnection() {
    const connectionId = form.getValues("id");
    if (connectionId) {
      testMutation.mutate(connectionId);
    }
  }

  const accountOptions: ResponsiveSelectOption[] = [
    { label: "None (select during import)", value: "" },
    ...accounts.map((account) => ({
      label: account.name,
      value: account.id,
    })),
  ];

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-8">
        <DialogHeader>
          <DialogTitle>
            {defaultValues?.id ? "Update IBKR Connection" : "Add IBKR Connection"}
          </DialogTitle>
          <DialogDescription>
            {defaultValues?.id
              ? "Update your Interactive Brokers Flex Query connection settings."
              : "Connect to Interactive Brokers using Flex Query Web Service."}
          </DialogDescription>
        </DialogHeader>

        <div className="grid gap-6 p-4">
          <FormField
            control={form.control}
            name="name"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Connection Name</FormLabel>
                <FormControl>
                  <Input placeholder="My IBKR Account" {...field} />
                </FormControl>
                <FormDescription>A descriptive name for this connection.</FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />

          <FormField
            control={form.control}
            name="queryId"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Flex Query ID</FormLabel>
                <FormControl>
                  <Input placeholder="123456" {...field} />
                </FormControl>
                <FormDescription>
                  Your IBKR Flex Query ID. See{" "}
                  <a
                    href="https://github.com/ni3do/wealthfolio/blob/main/docs/IBKR_FLEX_QUERY_SETUP.md"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-primary underline"
                  >
                    setup guide
                  </a>
                  .
                </FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />

          <FormField
            control={form.control}
            name="token"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Flex Query Token</FormLabel>
                <FormControl>
                  <Input type="password" placeholder="Enter token" {...field} />
                </FormControl>
                <FormDescription>
                  Your IBKR Web Service token. Stored securely in system keychain.
                </FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />

          <FormField
            control={form.control}
            name="accountId"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Link to Account (Optional)</FormLabel>
                <FormControl>
                  <ResponsiveSelect
                    options={accountOptions}
                    value={field.value || ""}
                    onValueChange={field.onChange}
                    placeholder="Select account"
                  />
                </FormControl>
                <FormDescription>
                  Link to a WealthFolio account for automatic import.
                </FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />

          <FormField
            control={form.control}
            name="syncInterval"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Sync Frequency</FormLabel>
                <FormControl>
                  <ResponsiveSelect
                    options={syncIntervalOptions}
                    value={field.value}
                    onValueChange={field.onChange}
                    placeholder="Select frequency"
                  />
                </FormControl>
                <FormDescription>How often to sync automatically.</FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />

          <FormField
            control={form.control}
            name="autoImport"
            render={({ field }) => (
              <FormItem className="flex flex-row items-center justify-between rounded-lg border p-4">
                <div className="space-y-0.5">
                  <FormLabel className="text-base">Auto Import</FormLabel>
                  <FormDescription>
                    Automatically import activities after sync (requires linked account).
                  </FormDescription>
                </div>
                <FormControl>
                  <Switch checked={field.value} onCheckedChange={field.onChange} />
                </FormControl>
              </FormItem>
            )}
          />
        </div>

        <DialogFooter className="gap-2">
          {defaultValues?.id && (
            <Button
              type="button"
              variant="outline"
              onClick={handleTestConnection}
              disabled={testMutation.isPending}
            >
              {testMutation.isPending ? (
                <>
                  <Icons.Spinner className="mr-2 h-4 w-4 animate-spin" />
                  Testing...
                </>
              ) : (
                <>
                  <Icons.CheckCircle className="mr-2 h-4 w-4" />
                  Test Connection
                </>
              )}
            </Button>
          )}
          <Button
            type="submit"
            disabled={createMutation.isPending || updateMutation.isPending}
          >
            {(createMutation.isPending || updateMutation.isPending) && (
              <Icons.Spinner className="mr-2 h-4 w-4 animate-spin" />
            )}
            {defaultValues?.id ? "Update Connection" : "Create Connection"}
          </Button>
        </DialogFooter>
      </form>
    </Form>
  );
}
