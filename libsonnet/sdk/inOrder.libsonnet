local annotation = 'kct.io/order';

local recursive(when, what, who, depth = 0) = if when(who) then what(depth, who) else std.mapWithKey(function(k, v) recursive(when, what, v, depth + 1), who);

local isObject(who) = std.objectHas(who, 'kind') && std.objectHas(who, 'apiVersion');

local annotate(with, who) = who + { metadata+: { annotations+: { [annotation]+: '/%s' % with } } };

local inOrder(order, who) =
  local allFields = std.sort(std.objectFields(who));
  local orderedFields = std.set(order);
  local missingFields = std.setDiff(allFields, orderedFields);

  local withIndex = std.mapWithIndex(function(i, f) { index: i, field: f }, order);
  local amount = std.length(order);

  local partiallyOrdered = std.foldl(
    function(acc, el) acc + {
      [el.field]+: recursive(
          isObject,
          function(depth, who) annotate('%s:%d:%d' % [el.field, depth, el.index], who),
          acc[el.field]
      )
    },
    withIndex,
    who
  );

  local totallyOrdered = std.foldl(
    function(acc, el) acc + {
      [el]+: recursive(
        isObject,
        function(depth, who) annotate('%s:%d:%d' % [el, depth, amount], who),
        acc[el]
      )
    },
    missingFields,
    partiallyOrdered
  );

  totallyOrdered;

inOrder
