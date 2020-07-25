package com.osohq.oso;

import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.lang.reflect.Field;
import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.util.*;
import java.util.stream.Collectors;
import org.json.JSONObject;
import org.json.JSONException;
import org.json.JSONArray;

public class Query implements Enumeration<HashMap<String, Object>> {
    private HashMap<String, Object> next;
    private Ffi.Query ffiQuery;
    private Host host;
    private Map<Long, Enumeration<Object>> calls;

    /**
     * Construct a new Query object.
     *
     * @param queryPtr Pointer to the FFI query instance.
     */
    public Query(Ffi.Query queryPtr, Host host) throws Exceptions.OsoException {
        this.ffiQuery = queryPtr;
        this.host = host;
        calls = new HashMap<Long, Enumeration<Object>>();
        next = nextResult();
    }

    @Override
    public boolean hasMoreElements() {
        return next != null;
    }

    @Override
    public HashMap<String, Object> nextElement() {
        HashMap<String, Object> ret = next;
        try {
            next = nextResult();
        } catch (Exception e) {
            throw new NoSuchElementException("Caused by: e.toString()");
        }
        return ret;
    }

    /**
     * Get all query results
     *
     * @return List of all query results (binding sets)
     */
    public List<HashMap<String, Object>> results() {
        List<HashMap<String, Object>> results = Collections.list(this);
        return results;
    }

    /**
     * Helper for `ExternalCall` query events
     *
     * @param attrName
     * @param jArgs
     * @param instanceId
     * @param callId
     * @throws Exceptions.OsoException
     */
    private void handleCall(String attrName, JSONArray jArgs, JSONObject polarInstance, long callId)
            throws Exceptions.OsoException {
        List<Object> args = host.polarListToJava(jArgs);
        registerCall(attrName, args, callId, polarInstance);
        String result;
        try {
            result = nextCallResult(callId).toString();
        } catch (NoSuchElementException e) {
            result = null;
        }
        ffiQuery.callResult(callId, result);
    }

    /**
     * Generate the next Query result
     *
     * @return
     * @throws Exceptions.OsoException
     */
    private HashMap<String, Object> nextResult() throws Exceptions.OsoException {
        while (true) {
            String eventStr = ffiQuery.nextEvent().get();
            String kind;
            JSONObject data;
            try {
                JSONObject event = new JSONObject(eventStr);
                kind = event.keys().next();
                data = event.getJSONObject(kind);
            } catch (JSONException e) {
                // TODO: this sucks, we should have a consistent serialization format
                kind = eventStr.replace("\"", "");
                data = null;
            }

            switch (kind) {
                case "Done":
                    return null;
                case "Result":
                    return host.polarDictToJava(data.getJSONObject("bindings"));
                case "MakeExternal":
                    Long id = data.getLong("instance_id");
                    if (host.hasInstance(id)) {
                        throw new Exceptions.DuplicateInstanceRegistrationError(id);
                    }
                    String clsName = data.getJSONObject("instance").getString("tag");
                    JSONObject jFields = data.getJSONObject("instance").getJSONObject("fields").getJSONObject("fields");
                    host.makeInstance(clsName, host.polarDictToJava(jFields), id);
                    break;
                case "ExternalCall":
                    long callId = data.getLong("call_id");
                    JSONObject polarInstance = data.getJSONObject("instance");
                    String attrName = data.getString("attribute");
                    JSONArray jArgs = data.getJSONArray("args");
                    handleCall(attrName, jArgs, polarInstance, callId);
                    break;
                case "ExternalIsa":
                    Long instanceId = data.getLong("instance_id");
                    callId = data.getLong("call_id");
                    String classTag = data.getString("class_tag");
                    int answer = host.isa(instanceId, classTag) ? 1 : 0;
                    ffiQuery.questionResult(callId, answer);
                    break;
                case "ExternalIsSubSpecializer":
                    instanceId = data.getLong("instance_id");
                    callId = data.getLong("call_id");
                    String leftTag = data.getString("left_class_tag");
                    String rightTag = data.getString("right_class_tag");
                    answer = host.subspecializer(instanceId, leftTag, rightTag) ? 1 : 0;
                    ffiQuery.questionResult(callId, answer);
                    break;
                case "Debug":
                    if (data.has("message")) {
                        String message = data.getString("message");
                        System.out.println(message);
                    }
                    BufferedReader br = new BufferedReader(new InputStreamReader(System.in));
                    System.out.print("> ");
                    try {
                        String input = br.readLine();
                        if (input == "")
                            input = " ";
                        String command = host.toPolarTerm(input).toString();
                        ffiQuery.debugCommand(command);
                    } catch (IOException e) {
                        throw new Exceptions.PolarRuntimeException("Caused by: " + e.getMessage());
                    }
                    break;
                default:
                    throw new Exceptions.PolarRuntimeException("Unhandled event type: " + kind);
            }
        }
    }

    /**
     * Register a Java method call, wrapping the result in an enumeration if it
     * isn't already done.
     *
     * @param attrName      Name of the method/attribute.
     * @param args          Method arguments.
     * @param callId        Call ID under which to register the call.
     * @param polarInstance JSONObject containing either an instance_id or an
     *                      instance of a built-in type.
     * @throws Exceptions.InvalidCallError
     */
    public void registerCall(String attrName, List<Object> args, long callId, JSONObject polarInstance)
            throws Exceptions.InvalidCallError, Exceptions.UnregisteredInstanceError,
            Exceptions.UnexpectedPolarTypeError {
        if (calls.containsKey(callId)) {
            return;
        }
        Object instance;
        if (polarInstance.getJSONObject("value").has("ExternalInstance")) {
            long instanceId = polarInstance.getJSONObject("value").getJSONObject("ExternalInstance")
                    .getLong("instance_id");
            instance = host.getInstance(instanceId);
        } else {
            instance = host.toJava(polarInstance);
        }

        // Get types of args to pass into `getMethod()`
        List<Class> argTypes = args.stream().map(a -> a.getClass()).collect(Collectors.toUnmodifiableList());
        Object result = null;
        Boolean isMethod = true;
        try {
            Class cls = instance instanceof Class ? (Class) instance : instance.getClass();
            try {
                Method method = cls.getMethod(attrName, argTypes.toArray(new Class[argTypes.size()]));
                result = method.invoke(instance, args.toArray());
            } catch (NoSuchMethodException e) {
                isMethod = false;
            }
            if (!isMethod) {
                try {
                    Field field = cls.getField(attrName);
                    result = field.get(instance);
                } catch (NoSuchFieldException e) {
                    throw new Exceptions.InvalidCallError(cls.getName(), attrName, argTypes);
                }
            }
        } catch (IllegalAccessException e) {
            throw new Exceptions.InvalidCallError("Caused by: " + e.toString());
        } catch (InvocationTargetException e) {
            throw new Exceptions.InvalidCallError("Caused by: " + e.toString());
        }
        Enumeration<Object> enumResult;
        if (result instanceof Enumeration) {
            // TODO: test this
            enumResult = (Enumeration<Object>) result;
        } else {
            enumResult = Collections.enumeration(new ArrayList<Object>(Arrays.asList(result)));
        }
        calls.put(callId, enumResult);

    }

    /**
     * Get cached Java method call result.
     *
     * @param callId
     * @return
     * @throws Exceptions.PolarRuntimeException
     */
    private Enumeration<Object> getCall(long callId) throws Exceptions.PolarRuntimeException {
        if (calls.containsKey(callId)) {
            return calls.get(callId);
        } else {
            throw new Exceptions.PolarRuntimeException("Unregistered call ID: " + callId);
        }

    }

    /**
     * Get the next JSONified Polar result of a cached method call (enumeration).
     *
     * @param callId
     * @return JSONObject
     * @throws NoSuchElementException
     * @throws Exceptions.OsoException
     */
    protected JSONObject nextCallResult(long callId) throws NoSuchElementException, Exceptions.OsoException {
        return host.toPolarTerm(getCall(callId).nextElement());
    }
}